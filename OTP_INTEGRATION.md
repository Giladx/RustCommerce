# 019 SMS OTP Integration

This document describes the OTP (One-Time Password) authentication system integrated with 019 SMS API.

## Overview

RustCommerce now includes a complete OTP authentication system using the 019 SMS service for Israeli phone numbers. This provides secure, phone-based authentication for customers.

## Features

✅ **Send OTP via 019 SMS API** - Sends verification codes to Israeli mobile numbers  
✅ **Verify OTP codes** - Validates codes with attempt tracking  
✅ **Automatic expiration** - Codes expire after configured time (default 5 minutes)  
✅ **Rate limiting** - Prevents spam (2-minute cooldown between requests)  
✅ **Attempt tracking** - Limits verification attempts (default 3 tries)  
✅ **Database persistence** - All verifications stored for audit trail  
✅ **Cleanup mechanism** - Automatic removal of expired OTP records

## API Endpoints

### Send OTP
```http
POST /api/otp/send
Content-Type: application/json

{
  "phone": "0501234567",
  "app_id": 1  // Optional, default is 1
}
```

**Response:**
```json
{
  "success": true,
  "message": "OTP sent successfully",
  "phone": "0501234567",
  "valid_until": "2026-01-12T11:30:00Z",
  "max_tries": 3
}
```

### Verify OTP
```http
POST /api/otp/verify
Content-Type: application/json

{
  "phone": "0501234567",
  "code": "448431",
  "app_id": 1  // Optional, default is 1
}
```

**Response (Success):**
```json
{
  "success": true,
  "verified": true,
  "phone": "0501234567",
  "verified_at": "2026-01-12T11:25:30Z"
}
```

**Response (Error):**
```json
{
  "error": "Invalid code. 2 attempts remaining."
}
```

## Configuration

Add these variables to your `.env` file:

```env
# 019 SMS OTP Configuration
OTP_USERNAME=your_019_username
OTP_SOURCE=YourBrand
```

### Configuration Options

| Variable | Description | Default |
|----------|-------------|---------|
| `OTP_USERNAME` | Your 019 SMS account username | Required |
| `OTP_SOURCE` | SMS sender name (shown to recipients) | `***` |
| `default_app_id` | Application ID for multi-app setups | `1` |
| `default_max_tries` | Maximum verification attempts | `3` |
| `default_valid_minutes` | Code validity period in minutes | `5` |
| `default_text` | Custom SMS text (must contain `[code]`) | Hebrew default |

## Database Schema

```sql
CREATE TABLE otp_verifications (
    id UUID PRIMARY KEY,
    numeric_id SERIAL UNIQUE,
    phone VARCHAR(20) NOT NULL,
    code VARCHAR(10) NOT NULL,
    app_id INTEGER NOT NULL DEFAULT 1,
    max_tries INTEGER NOT NULL DEFAULT 3,
    tries_count INTEGER NOT NULL DEFAULT 0,
    valid_until TIMESTAMPTZ NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ
);
```

## Phone Number Format

The system accepts multiple phone formats and normalizes them:

- `0501234567` ✅ (preferred)
- `501234567` ✅ (auto-prefixed with 0)
- `05-123-4567` ❌ (remove dashes first)

## Usage Examples

### Customer Registration Flow

```javascript
// Step 1: Request OTP
const sendResponse = await fetch('/api/otp/send', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ phone: '0501234567' })
});

const sendData = await sendResponse.json();
console.log(sendData.message); // "OTP sent successfully"

// Step 2: User enters code from SMS

// Step 3: Verify OTP
const verifyResponse = await fetch('/api/otp/verify', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ 
    phone: '0501234567',
    code: '448431'
  })
});

const verifyData = await verifyResponse.json();
if (verifyData.verified) {
  // Proceed with registration/login
  console.log('Phone verified!');
}
```

### Multi-App Support

If you have multiple applications (web, mobile, etc.):

```javascript
// Web app
fetch('/api/otp/send', {
  method: 'POST',
  body: JSON.stringify({ 
    phone: '0501234567',
    app_id: 1  // Web
  })
});

// Mobile app
fetch('/api/otp/send', {
  method: 'POST',
  body: JSON.stringify({ 
    phone: '0501234567',
    app_id: 2  // Mobile
  })
});
```

## Error Handling

| Error Message | Cause | Solution |
|--------------|-------|----------|
| "OTP already sent. Please wait..." | Request within 2-minute cooldown | Wait before requesting new code |
| "Invalid code. X attempts remaining." | Wrong code entered | Try again with correct code |
| "OTP code has expired..." | Code older than 5 minutes | Request new code |
| "Maximum verification attempts exceeded" | 3 wrong attempts | Request new code |
| "This OTP code has already been verified" | Code already used | Code is one-time use only |

## Security Features

1. **Time-based expiration**: Codes expire after 5 minutes
2. **Attempt limiting**: Maximum 3 verification attempts per code
3. **Rate limiting**: 2-minute cooldown between SMS sends
4. **One-time use**: Codes cannot be reused after verification
5. **Database audit trail**: All attempts logged for security analysis

## Cleanup

Expired OTP records are automatically cleaned up:

```rust
// Run periodically (e.g., daily cron job)
use crate::api::otp_service::cleanup_expired_otps;

let deleted = cleanup_expired_otps(&pool).await?;
println!("Cleaned up {} expired OTP records", deleted);
```

## Testing

### Development/Testing Mode

For development, you can check the database directly to see the generated code:

```sql
SELECT phone, code, created_at, valid_until, verified
FROM otp_verifications
WHERE phone = '0501234567'
ORDER BY created_at DESC
LIMIT 1;
```

### Test Phone Numbers

For testing without actual SMS:
1. Modify `OTP_USERNAME` in `.env` to your test account
2. Use 019's test phone numbers (check their documentation)
3. Codes will be logged in the database but not sent via SMS

## Integration with Customer Authentication

After successful OTP verification, you can:

1. Create or link to customer account:
```rust
// Get customer by phone or create new
let customer = get_or_create_customer_by_phone(&pool, &verified_phone).await?;

// Create session token
let session_token = create_session_token(customer.id);
```

2. Issue JWT token for API access:
```rust
let claims = Claims {
    sub: customer.id.to_string(),
    phone: verified_phone,
    exp: (Utc::now() + Duration::hours(24)).timestamp(),
};
let token = encode(&Header::default(), &claims, &encoding_key)?;
```

## Troubleshooting

### SMS not received

1. Check 019 account balance
2. Verify `OTP_USERNAME` is correct
3. Check phone number format
4. Review 019 API logs

### "Failed to send OTP" error

1. Verify 019 API credentials
2. Check network connectivity
3. Ensure 019 API is not rate-limiting your account
4. Check response status in logs

### Database connection errors

1. Verify `DATABASE_URL` in `.env`
2. Ensure migrations have run: `sqlx migrate run`
3. Check database permissions

## Production Checklist

- [ ] Configure real 019 SMS account credentials
- [ ] Set up proper SMS sender name (`OTP_SOURCE`)
- [ ] Customize SMS text in Hebrew
- [ ] Set up OTP cleanup cron job
- [ ] Configure rate limiting per IP
- [ ] Add monitoring for OTP send failures
- [ ] Set up alerts for suspicious activity
- [ ] Test all error scenarios
- [ ] Review security settings

## API Response Codes

| Status | Description |
|--------|-------------|
| 200 | Success |
| 400 | Bad request (invalid phone, already sent, etc.) |
| 500 | Server error (database, 019 API failure) |

## Support

For 019 SMS API issues, contact: https://019sms.co.il/support  
For RustCommerce OTP issues, check the logs or create an issue.

---

**Last Updated**: 2026-01-12  
**Version**: 1.0.0

-- Create OTP verification table for 019 SMS integration
CREATE TABLE IF NOT EXISTS otp_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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

-- Add numeric ID
ALTER TABLE otp_verifications ADD COLUMN IF NOT EXISTS numeric_id SERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_otp_verifications_numeric_id ON otp_verifications(numeric_id);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_otp_phone ON otp_verifications(phone);
CREATE INDEX IF NOT EXISTS idx_otp_phone_app ON otp_verifications(phone, app_id);
CREATE INDEX IF NOT EXISTS idx_otp_verified ON otp_verifications(verified);
CREATE INDEX IF NOT EXISTS idx_otp_valid_until ON otp_verifications(valid_until);
CREATE INDEX IF NOT EXISTS idx_otp_created_at ON otp_verifications(created_at DESC);

-- Composite index for active verifications
CREATE INDEX IF NOT EXISTS idx_otp_active ON otp_verifications(phone, app_id, verified, valid_until) 
WHERE verified = FALSE;

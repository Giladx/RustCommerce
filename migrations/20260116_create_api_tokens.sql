-- API Tokens for authentication
CREATE TABLE IF NOT EXISTS api_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    token VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

-- Index for fast token lookups
CREATE INDEX IF NOT EXISTS idx_api_tokens_token ON api_tokens(token) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_api_tokens_created ON api_tokens(created_at DESC);

-- Insert a default demo token
INSERT INTO api_tokens (name, token, description, is_active)
VALUES ('Demo Token', 'demo-token-12345', 'Default API token for testing', true)
ON CONFLICT (token) DO NOTHING;

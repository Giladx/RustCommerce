CREATE TABLE IF NOT EXISTS system_settings (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO system_settings (key, value, description)
VALUES ('api_auth_enabled', 'false', 'Enforce API token authentication for all /api routes')
ON CONFLICT (key) DO NOTHING;

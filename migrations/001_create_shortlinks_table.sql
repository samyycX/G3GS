-- Create shortlinks table
CREATE TABLE IF NOT EXISTS shortlinks (
    id BIGSERIAL PRIMARY KEY,
    original_url TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    expires_at TIMESTAMPTZ,
    access_count BIGINT DEFAULT 0 NOT NULL,
    UNIQUE(original_url)
);

-- Create index on expires_at for cleanup queries
CREATE INDEX IF NOT EXISTS idx_shortlinks_expires_at ON shortlinks(expires_at);

-- Create access_logs table for tracking visits
CREATE TABLE IF NOT EXISTS access_logs (
    id BIGSERIAL PRIMARY KEY,
    shortlink_id BIGINT NOT NULL,
    accessed_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    ip_address INET,
    user_agent TEXT,
    FOREIGN KEY (shortlink_id) REFERENCES shortlinks(id) ON DELETE CASCADE
);

-- Create index on shortlink_id for access_logs
CREATE INDEX IF NOT EXISTS idx_access_logs_shortlink_id ON access_logs(shortlink_id);
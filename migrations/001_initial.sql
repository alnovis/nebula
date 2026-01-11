-- Initial database schema
-- Note: Posts and projects are stored as markdown files, not in DB
-- This table is for analytics and future features

CREATE TABLE IF NOT EXISTS page_views (
    id SERIAL PRIMARY KEY,
    path VARCHAR(512) NOT NULL,
    referrer VARCHAR(1024),
    user_agent VARCHAR(512),
    ip_hash VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_page_views_path ON page_views(path);
CREATE INDEX idx_page_views_created_at ON page_views(created_at);

-- For future: newsletter subscribers
CREATE TABLE IF NOT EXISTS subscribers (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    confirmed BOOLEAN NOT NULL DEFAULT FALSE,
    confirm_token VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ
);

CREATE INDEX idx_subscribers_email ON subscribers(email);

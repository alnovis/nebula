-- Behavioral analytics tables for tracking user navigation, events, and sessions.
-- The existing page_views table (001_initial.sql) is left untouched.

-- Server-side page navigation events (written by middleware)
CREATE TABLE IF NOT EXISTS page_events (
    id BIGSERIAL PRIMARY KEY,
    session_id VARCHAR(64) NOT NULL,
    path VARCHAR(512) NOT NULL,
    prev_path VARCHAR(512),
    referrer VARCHAR(1024),
    user_agent VARCHAR(512),
    ip_hash VARCHAR(64) NOT NULL,
    country VARCHAR(2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_page_events_session ON page_events(session_id);
CREATE INDEX idx_page_events_path ON page_events(path, created_at);
CREATE INDEX idx_page_events_created_at ON page_events(created_at);
CREATE INDEX idx_page_events_referrer ON page_events(referrer) WHERE referrer IS NOT NULL;

-- Client-side behavioral events (written by JS beacon)
CREATE TABLE IF NOT EXISTS client_events (
    id BIGSERIAL PRIMARY KEY,
    session_id VARCHAR(64) NOT NULL,
    path VARCHAR(512) NOT NULL,
    event_type VARCHAR(32) NOT NULL,
    event_data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_client_events_session ON client_events(session_id);
CREATE INDEX idx_client_events_type ON client_events(event_type, path);
CREATE INDEX idx_client_events_created_at ON client_events(created_at);

-- Session metadata
CREATE TABLE IF NOT EXISTS analytics_sessions (
    id VARCHAR(64) PRIMARY KEY,
    first_path VARCHAR(512) NOT NULL,
    first_referrer VARCHAR(1024),
    ip_hash VARCHAR(64) NOT NULL,
    user_agent VARCHAR(512),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_analytics_sessions_started ON analytics_sessions(started_at);
CREATE INDEX idx_analytics_sessions_referrer ON analytics_sessions(first_referrer) WHERE first_referrer IS NOT NULL;

-- GeoIP lookup table (populated by nightly job from RIR delegation files)
CREATE TABLE IF NOT EXISTS geoip_ranges (
    id SERIAL PRIMARY KEY,
    network CIDR NOT NULL,
    country VARCHAR(2) NOT NULL
);

CREATE INDEX idx_geoip_network ON geoip_ranges USING gist (network inet_ops);

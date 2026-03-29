-- Monthly aggregates (populated before cleanup, preserves historical trends)
CREATE TABLE IF NOT EXISTS analytics_monthly (
    id SERIAL PRIMARY KEY,
    month DATE NOT NULL,
    path VARCHAR(512) NOT NULL,
    page_views BIGINT NOT NULL DEFAULT 0,
    unique_sessions BIGINT NOT NULL DEFAULT 0,
    avg_scroll_depth REAL,
    avg_visibility_seconds REAL,
    country VARCHAR(2),
    UNIQUE (month, path, country)
);

CREATE INDEX idx_analytics_monthly_month ON analytics_monthly(month);

CREATE TABLE IF NOT EXISTS analytics_monthly_referrers (
    id SERIAL PRIMARY KEY,
    month DATE NOT NULL,
    referrer VARCHAR(1024) NOT NULL,
    sessions BIGINT NOT NULL DEFAULT 0,
    UNIQUE (month, referrer)
);

CREATE INDEX idx_analytics_monthly_ref_month ON analytics_monthly_referrers(month);

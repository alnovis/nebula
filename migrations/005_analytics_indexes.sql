-- Composite index for human session filtering and article metrics
CREATE INDEX IF NOT EXISTS idx_client_events_session_type ON client_events(session_id, event_type);

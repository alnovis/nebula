-- Delete raw analytics recorded from given IP addresses, for example after adding them
-- to ANALYTICS_EXCLUDE_IPS. Whole sessions are deleted: page events, client events and
-- the session row. Monthly aggregates (analytics_monthly*) are not touched.
--
-- Only the client address is stored, as ip_hash = first 16 bytes of
-- sha256(ip || 'nebula-analytics-' || UTC date), hex (views::hash_ip_daily), so the
-- hashes are recomputed for every day of retained data. Traffic that came through a
-- listed proxy carries the original client's hash and cannot be found this way.
--
-- Without apply=1 everything runs in a transaction that is rolled back: the DELETE
-- counts are printed and nothing changes.
--
--   psql -v ips="'203.0.113.9','198.51.100.7'" -f scripts/analytics-exclude-ips.sql
--   psql -v ips="'203.0.113.9','198.51.100.7'" -v apply=1 -f scripts/analytics-exclude-ips.sql

\set ON_ERROR_STOP on

\if :{?ips}
\else
\echo 'usage: psql -v ips="''1.2.3.4'',''5.6.7.8''" [-v apply=1] -f scripts/analytics-exclude-ips.sql'
\quit
\endif

\if :{?apply}
\else
\set apply 0
\endif

SET TIME ZONE 'UTC';

BEGIN;

CREATE TEMP TABLE excluded_hashes ON COMMIT DROP AS
SELECT encode(substring(sha256(convert_to(ip || 'nebula-analytics-' || d::date::text, 'UTF8')) FROM 1 FOR 16), 'hex') AS ip_hash
FROM unnest(ARRAY[:ips]::text[]) AS ip,
     generate_series(
         (SELECT COALESCE(min(started_at), now()) FROM analytics_sessions)::date,
         current_date,
         interval '1 day'
     ) AS d;

CREATE TEMP TABLE excluded_sessions ON COMMIT DROP AS
SELECT session_id FROM page_events WHERE ip_hash IN (SELECT ip_hash FROM excluded_hashes)
UNION
SELECT id FROM analytics_sessions WHERE ip_hash IN (SELECT ip_hash FROM excluded_hashes);

SELECT count(*) AS sessions_matched FROM excluded_sessions;

DELETE FROM client_events WHERE session_id IN (SELECT session_id FROM excluded_sessions);
DELETE FROM page_events WHERE session_id IN (SELECT session_id FROM excluded_sessions);
DELETE FROM analytics_sessions WHERE id IN (SELECT session_id FROM excluded_sessions);

\if :apply
COMMIT;
\echo 'deleted'
\else
ROLLBACK;
\echo 'dry run, nothing deleted; add -v apply=1 to delete'
\endif

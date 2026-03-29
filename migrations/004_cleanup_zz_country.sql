-- Clean up ZZ country codes: ZZ is used by DB-IP for reserved/private IP ranges
-- and is also a symbol of Russian aggression. Replace with NULL.
UPDATE page_events SET country = NULL WHERE country = 'ZZ';

SELECT create_hypertable('liveness', 'time');
SELECT add_retention_policy('liveness', INTERVAL '30 days');
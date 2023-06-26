CREATE TABLE liveness
(
    endpoint_id_fk BIGSERIAL   NOT NULL REFERENCES endpoint (id) ON DELETE CASCADE,
    time           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_alive       BOOLEAN     NOT NULL,
    error          TEXT
);

CREATE INDEX liveness_endpoint_id_fk_idx ON liveness (endpoint_id_fk);
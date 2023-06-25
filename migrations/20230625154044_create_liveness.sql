CREATE TABLE liveness
(
    id             BIGSERIAL PRIMARY KEY,
    endpoint_id_fk BIGSERIAL   NOT NULL REFERENCES endpoint (id) ON DELETE CASCADE,
    time           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_alive       BOOLEAN     NOT NULL,
    error          TEXT
);
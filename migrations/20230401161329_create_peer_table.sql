CREATE TABLE peer
(
    id          BIGSERIAL PRIMARY KEY,
    type        TEXT        NOT NULL, -- 'seed' or 'persistent'
    address     TEXT        NOT NULL,
    is_alive    BOOLEAN     NOT NULL DEFAULT TRUE,
    chain_id_fk BIGINT      NOT NULL REFERENCES chain (id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE peer
(
    id          SERIAL PRIMARY KEY,
    type        TEXT        NOT NULL, -- 'seed' or 'persistent'
    address     TEXT        NOT NULL,
    provider    TEXT        NOT NULL DEFAULT 'unknown',
    is_alive    BOOLEAN     NOT NULL,
    chain_id_fk INTEGER     NOT NULL REFERENCES chain (id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE chain
(
    id         BIGSERIAL PRIMARY KEY,
    name       TEXT        NOT NULL,
    network    TEXT        NOT NULL,
    chain_data jsonb       NOT NULL,
    asset_data jsonb       NOT NULL,
    commit     TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

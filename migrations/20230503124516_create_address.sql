CREATE TABLE address (
    id BIGSERIAL PRIMARY KEY,
    url TEXT NOT NULL,
    kind TEXT NOT NULL, -- 'seed', 'peer', 'rpc', 'lcd', 'grpc', etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX address_url_kind_idx ON address (kind, url);

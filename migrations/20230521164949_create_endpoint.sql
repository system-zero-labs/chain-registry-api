CREATE TYPE endpoint_kind AS ENUM (
    'seed',
    'peer',
    'rpc',
    'rest',
    'grpc'
);

CREATE TABLE endpoint
(
    id         BIGSERIAL PRIMARY KEY,
    address    TEXT          NOT NULL,
    kind       endpoint_kind NOT NULL,
    provider   TEXT          NOT NULL DEFAULT 'Unknown',
    created_at TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX endpoint_address_kind_idx ON endpoint (address, kind);

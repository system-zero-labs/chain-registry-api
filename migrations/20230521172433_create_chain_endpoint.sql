CREATE TABLE chain_endpoint
(
    chain_id_fk BIGSERIAL NOT NULL REFERENCES chain (id) ON DELETE CASCADE,
    endpoint_id_fk BIGSERIAL NOT NULL REFERENCES endpoint (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX chain_endpoint_chain_id_fk_endpoint_id_fk_idx ON chain_endpoint (chain_id_fk, endpoint_id_fk);

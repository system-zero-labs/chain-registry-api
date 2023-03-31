CREATE
OR REPLACE FUNCTION set_created_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.created_at
:= NOW();
RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE TYPE network AS ENUM ('mainnet', 'testnet');

CREATE TABLE chain
(
    id         uuid        NOT NULL,
    PRIMARY KEY (id),
    name       TEXT        NOT NULL,
    network    network     NOT NULL,
    raw_data   jsonb       NOT NULL,
    created_at timestamptz NOT NULL
);

CREATE TRIGGER chain_created_at_trigger
    BEFORE INSERT
    ON chain
    FOR EACH ROW EXECUTE FUNCTION set_created_at();
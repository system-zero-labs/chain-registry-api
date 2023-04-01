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

CREATE TABLE chain
(
    id         SERIAL PRIMARY KEY,
    name       TEXT        NOT NULL,
    network    TEXT        NOT NULL,
    chain_data jsonb       NOT NULL,
    asset_data jsonb       NOT NULL,
    created_at timestamptz NOT NULL
);

CREATE TRIGGER chain_created_at_trigger
    BEFORE INSERT
    ON chain
    FOR EACH ROW EXECUTE FUNCTION set_created_at();
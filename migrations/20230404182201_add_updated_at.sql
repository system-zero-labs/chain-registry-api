ALTER TABLE chain ADD COLUMN updated_at timestamptz NOT NULL DEFAULT now();
ALTER TABLE peer ADD COLUMN updated_at timestamptz NOT NULL DEFAULT now();

CREATE OR REPLACE FUNCTION set_updated_at()
    RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER chain_set_updated_at
    BEFORE UPDATE ON chain
    FOR EACH ROW
    EXECUTE PROCEDURE set_updated_at();

CREATE TRIGGER peer_set_updated_at
    BEFORE UPDATE ON peer
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

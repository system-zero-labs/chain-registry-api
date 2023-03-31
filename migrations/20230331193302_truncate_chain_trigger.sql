CREATE OR REPLACE FUNCTION truncate_chains()
    RETURNS TRIGGER AS
$$
BEGIN
    DELETE
    FROM chain
    WHERE id NOT IN (SELECT id
                     FROM chain
                     ORDER BY created_at DESC
                     LIMIT 10000);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER limit_chain_rows
    AFTER INSERT
    ON chain
    FOR EACH ROW
EXECUTE FUNCTION truncate_chains();
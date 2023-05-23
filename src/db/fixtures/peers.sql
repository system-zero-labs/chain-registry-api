INSERT INTO chain (id, created_at, name, network, commit, asset_data, chain_data)
VALUES (1,
        NOW(),
        'cosmoshub',
        'mainnet',
        'previous_commit',
        '{}',
        '{}');

INSERT INTO endpoint (id, address, chain_id, kind, provider)
VALUES (1,
        'peer1@localhost:26656',
        'cosmoshub-4',
        'peer',
        'Company1');

INSERT INTO endpoint (id, address, chain_id, kind, provider)
VALUES (2,
        'peer2@localhost:26656',
        'cosmoshub-4',
        'peer',
        'Company2');

INSERT INTO endpoint (id, address, chain_id, kind, provider)
VALUES (3,
        'peer3@localhost:26656',
        'cosmoshub-4',
        'peer',
        'Company1');

INSERT INTO endpoint (id, address, chain_id, kind, provider)
VALUES (4,
        'seed1@localhost:26656',
        'cosmoshub-4',
        'seed',
        'Company1');

INSERT INTO chain_endpoint (chain_id_fk, endpoint_id_fk)
VALUES (1, 1), (1, 2), (1, 3), (1, 4);
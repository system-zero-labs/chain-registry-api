INSERT INTO chain (created_at, name, network, commit, asset_data, chain_data)
VALUES (NOW() - interval '3 hours',
        'cosmoshub',
        'mainnet',
        'commit1',
        '{}',
        '{}');

INSERT INTO chain (created_at, name, network, commit, asset_data, chain_data)
VALUES (NOW() - interval '2 hours',
        'cosmoshub',
        'mainnet',
        'commit2',
        '{}',
        '{}');

INSERT INTO chain (created_at, name, network, commit, asset_data, chain_data)
VALUES (NOW() - interval '1 hour',
        'cosmoshub',
        'mainnet',
        'commit3',
        '{}',
        '{}');

INSERT INTO chain (created_at, name, network, commit, asset_data, chain_data)
VALUES (NOW(),
        'cosmoshub',
        'mainnet',
        'commit4',
        '{}',
        '{}');

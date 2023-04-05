-- old chain
INSERT INTO chain (id, name, network, commit, asset_data, chain_data, created_at)
VALUES (1,
        'cosmoshub',
        'mainnet',
        'old_commit',
        '{}',
               -- cat tmp-chain-registry/cosmoshub/chain.json | jq -c | pbcopy
        '{}',
        now() - interval '1 hour');

-- new chain
INSERT INTO chain (id, name, network, commit, asset_data, chain_data)
VALUES (2,
        'cosmoshub',
        'mainnet',
        'new_commit',
        '{}',
           -- cat tmp-chain-registry/cosmoshub/chain.json | jq -c | pbcopy
        '{}');

INSERT INTO peer (chain_id_fk, type, address)
VALUES (2, 'seed', 'abc123@public-seed-node.com:26656');
INSERT INTO peer (chain_id_fk, type, address)
VALUES (2, 'persistent', 'efg987@public-persistent.com:26656');


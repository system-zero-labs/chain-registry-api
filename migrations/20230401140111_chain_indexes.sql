CREATE INDEX chain_name_idx ON chain (name);
CREATE INDEX chain_network_idx ON chain (network);
CREATE UNIQUE INDEX chain_network_name_commit_idx ON chain (network, name, commit);

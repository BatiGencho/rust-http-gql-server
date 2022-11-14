-- Your SQL goes here

CREATE TABLE if not exists block_pointer (
  id UUID,
  processed_at TIMESTAMP NOT NULL,
  last_processed_block_hash VARCHAR NOT NULL UNIQUE,
  last_processed_block_number BIGINT NOT NULL UNIQUE,
  PRIMARY KEY (id)
)
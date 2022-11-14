-- Your SQL goes here

CREATE TABLE if not exists sc_events (
  id UUID,
  processed_at TIMESTAMP NOT NULL,
  account_id VARCHAR NOT NULL,
  method_name VARCHAR NOT NULL,
  tx_hash VARCHAR NOT NULL,
  logs JSON NOT NULL,
  PRIMARY KEY (id)
)
-- Your SQL goes here

CREATE TABLE if not exists users (
  id UUID,
  name VARCHAR(255),
  username VARCHAR(255) NOT NULL UNIQUE,
  phone_number VARCHAR(50),
  email VARCHAR,
  password VARCHAR,
  encrypted_secret_key VARCHAR,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  wallet_id VARCHAR NOT NULL UNIQUE,
  wallet_balance VARCHAR NOT NULL,
  user_type SMALLINT NOT NULL DEFAULT 0,
  user_status SMALLINT NOT NULL DEFAULT 0,
  PRIMARY KEY (id)
)
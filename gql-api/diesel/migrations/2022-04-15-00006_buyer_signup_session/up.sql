-- Your SQL goes here

CREATE TABLE if not exists buyer_signup_sessions (
  id UUID,
  created_at TIMESTAMP NOT NULL,
  verification_code VARCHAR NOT NULL UNIQUE,
  phone_number VARCHAR,
  is_verified BOOLEAN NOT NULL DEFAULT 'f',
  PRIMARY KEY (id)
)
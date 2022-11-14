-- Your SQL goes here

CREATE TABLE if not exists sessions (
  id UUID,
  expires_at TIMESTAMP NOT NULL,
  login_code VARCHAR NOT NULL UNIQUE,
  is_used BOOLEAN NOT NULL DEFAULT 'f',
  user_id UUID,
  PRIMARY KEY (id)
)
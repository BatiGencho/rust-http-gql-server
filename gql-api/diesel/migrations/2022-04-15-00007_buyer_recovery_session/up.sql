-- Your SQL goes here

CREATE TABLE if not exists buyer_recovery_sessions (
  id UUID,
  created_at TIMESTAMP NOT NULL,
  recovery_code VARCHAR NOT NULL UNIQUE,
  phone_number VARCHAR,
  is_recovered BOOLEAN NOT NULL DEFAULT 'f',
  created_by_user UUID NOT NULL REFERENCES public.users (id) ON DELETE CASCADE,
  PRIMARY KEY (id)
)
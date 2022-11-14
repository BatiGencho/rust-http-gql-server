-- Your SQL goes here

CREATE TABLE if not exists events (
  id UUID,
  event_name VARCHAR NOT NULL,
  event_slug VARCHAR NOT NULL,
  start_date TIMESTAMP,
  end_date TIMESTAMP,
  entry_time TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  description VARCHAR,
  is_virtual BOOLEAN DEFAULT 'f',
  is_featured BOOLEAN DEFAULT 'f',
  venue_name VARCHAR,
  venue_location VARCHAR,
  cover_photo_url VARCHAR,
  thumbnail_url VARCHAR,
  event_status SMALLINT DEFAULT 0,
  created_by_user UUID REFERENCES public.users (id) ON DELETE CASCADE,
  PRIMARY KEY (id),
  UNIQUE (event_name, event_slug)
)
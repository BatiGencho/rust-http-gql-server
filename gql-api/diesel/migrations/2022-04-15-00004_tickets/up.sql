-- Your SQL goes here


CREATE TABLE if not exists tickets (
  id UUID,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  ticket_name VARCHAR NOT NULL,
  ticket_slug VARCHAR NOT NULL,
  description VARCHAR,
  price VARCHAR,
  max_release_price VARCHAR,
  quantity_available INTEGER DEFAULT 0,
  min_purchase_quantity INTEGER DEFAULT 0,
  max_purchase_quantity INTEGER DEFAULT 0,
  allow_transfers BOOLEAN DEFAULT 'f',
  event_id UUID REFERENCES public.events (id) ON DELETE CASCADE,
  PRIMARY KEY (id),
  UNIQUE (ticket_name, ticket_slug, event_id)
)
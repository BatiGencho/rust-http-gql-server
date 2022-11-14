-- Your SQL goes here

CREATE TABLE if not exists ticket_reservations (
  id UUID,
  created_at TIMESTAMP NOT NULL,
  verification_code VARCHAR NOT NULL,
  event_id UUID REFERENCES public.events (id) ON DELETE CASCADE,
  ticket_id UUID REFERENCES public.tickets (id) ON DELETE CASCADE,
  user_id UUID REFERENCES public.users (id) ON DELETE CASCADE,
  PRIMARY KEY (id),
  UNIQUE (verification_code, event_id, ticket_id, user_id)
)
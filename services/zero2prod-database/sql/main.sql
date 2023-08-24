DO $$
BEGIN
  CREATE ROLE bob WITH LOGIN PASSWORD 'secret';
  EXCEPTION WHEN DUPLICATE_OBJECT THEN
  RAISE NOTICE 'Not creating role ''bob'' -- it already exists';
END
$$;

DROP SCHEMA IF EXISTS main CASCADE;
CREATE SCHEMA main AUTHORIZATION bob;
GRANT ALL ON SCHEMA main to bob;
SET SEARCH_PATH = main;


CREATE TYPE main.subscription_status AS ENUM (
    'pending_confirmation',
    'confirmed'
);

ALTER TYPE main.subscription_status OWNER TO bob;

CREATE TABLE main.subscriptions (
    id uuid PRIMARY KEY NOT NULL,
    email text UNIQUE NOT NULL,
    username text NOT NULL,
    subscribed_at timestamp with time zone NOT NULL,
    status main.subscription_status NOT NULL
);

ALTER TABLE main.subscriptions OWNER TO bob;

CREATE TABLE main.subscription_tokens (
    subscription_token text PRIMARY KEY NOT NULL,
    subscriber_id uuid NOT NULL,
    CONSTRAINT fk_subscription_tokens_subscriber_id FOREIGN KEY (subscriber_id) REFERENCES subscriptions(id)
);

ALTER TABLE main.subscription_tokens OWNER TO bob;

CREATE TABLE main.users (
    id uuid PRIMARY KEY NOT NULL,
    username text UNIQUE NOT NULL,
    password_hash text NOT NULL,
    email text UNIQUE NOT NULL
);

ALTER TABLE main.users OWNER TO bob;

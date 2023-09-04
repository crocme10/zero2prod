CREATE TYPE subscription_status AS ENUM (
    'pending_confirmation',
    'confirmed'
);

CREATE TABLE subscriptions (
    id uuid PRIMARY KEY NOT NULL,
    email text UNIQUE NOT NULL,
    username text NOT NULL,
    subscribed_at timestamp with time zone NOT NULL,
    status subscription_status NOT NULL
);

CREATE TABLE subscription_tokens (
    subscription_token text PRIMARY KEY NOT NULL,
    subscriber_id uuid NOT NULL,
    CONSTRAINT fk_subscription_tokens_subscriber_id FOREIGN KEY (subscriber_id) REFERENCES subscriptions(id)
);

CREATE TABLE users (
    id uuid PRIMARY KEY NOT NULL,
    username text UNIQUE NOT NULL,
    password_hash text NOT NULL,
    email text UNIQUE NOT NULL
);

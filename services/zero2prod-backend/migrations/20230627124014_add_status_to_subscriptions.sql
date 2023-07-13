CREATE TYPE subscription_status AS ENUM ('pending_confirmation');
ALTER TABLE subscriptions ADD COLUMN status subscription_status NULL;

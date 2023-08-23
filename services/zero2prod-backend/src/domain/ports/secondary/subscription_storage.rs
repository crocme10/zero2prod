use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{ConfirmedSubscriber, NewSubscription, Subscription};
use super::Error;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SubscriptionStorage {
    /// Store a new subscription, and a token, and return the subscription
    async fn create_subscription_and_store_token(
        &self,
        subscription: &NewSubscription,
        token: &str,
    ) -> Result<Subscription, Error>;

    async fn get_subscription_by_email(&self, email: &str) -> Result<Option<Subscription>, Error>;

    async fn get_subscriber_id_by_token(&self, token: &str) -> Result<Option<Uuid>, Error>;

    async fn get_token_by_subscriber_id(&self, id: &Uuid) -> Result<Option<String>, Error>;

    /// Modify the status of the subscriber identified by id to 'confirmed'
    async fn confirm_subscriber_by_id_and_delete_token(&self, id: &Uuid) -> Result<(), Error>;

    /// Delete a previously stored token identified by a subscriber_id
    async fn delete_confirmation_token(&self, id: &Uuid) -> Result<(), Error>;

    async fn get_confirmed_subscribers_email(&self) -> Result<Vec<ConfirmedSubscriber>, Error>;
}

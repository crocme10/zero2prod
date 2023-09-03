use async_trait::async_trait;
use chrono::Utc;
use common::err_context::ErrorContextExt;
use std::str::FromStr;
use uuid::Uuid;

use super::PostgresStorage;
use crate::domain::{
    ports::secondary::SubscriptionError, ports::secondary::SubscriptionStorage,
    ConfirmedSubscriber, NewSubscription, SubscriberEmail, SubscriberName,
    Subscription, SubscriptionStatus,
};

#[async_trait]
impl SubscriptionStorage for PostgresStorage {
    #[tracing::instrument(name = "Storing a new subscription in postgres")]
    async fn create_subscription_and_store_token(
        &self,
        new_subscription: &NewSubscription,
        token: &str,
    ) -> Result<Subscription, SubscriptionError> {
        // FIXME The following two statements should be part of a transaction.
        // But I'm not sure how to bring in a Transaction from my Exec?
        // Side stepping this issue for now, as maybe sqlx will bring a solution
        // soon.
        let id = Uuid::new_v4();
        // FIXME Use a RETURNING clause instead of using a subsequent SELECT
        sqlx::query!(
        r#"INSERT INTO main.subscriptions (id, email, username, subscribed_at, status) VALUES ($1, $2, $3, $4, $5)"#,
        id,
        new_subscription.email.as_ref(),
        new_subscription.username.as_ref(),
        Utc::now(),
        SubscriptionStatus::PendingConfirmation as SubscriptionStatus,
        )
        .execute(&self.pool)
        .await
        .context(format!(
                "Could not store new subscription for {}", new_subscription.username.as_ref()
                ))?;

        sqlx::query!(
            r#"INSERT INTO main.subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)"#,
            token, id
        )
        .execute(&self.pool)
        .await
        .context(format!("Could not store subscription token for subscriber id {id}"))?;
        let saved = sqlx::query!(
            r#"SELECT id, email, username, status::text FROM main.subscriptions WHERE id = $1"#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .context(format!("Could not get subscription for {id}"))?;
        let username =
            SubscriberName::parse(saved.username).map_err(|err| SubscriptionError::Validation {
                context: format!("Invalid username stored in the database: {err}"),
            })?;
        let email =
            SubscriberEmail::parse(saved.email).map_err(|err| SubscriptionError::Validation {
                context: format!("Invalid email stored in the database: {err}"),
            })?;
        let status =
            SubscriptionStatus::from_str(&saved.status.unwrap_or_default()).map_err(|err| {
                SubscriptionError::Validation {
                    context: format!("Invalid status stored in the database: {err}"),
                }
            })?;
        Ok(Subscription {
            id: saved.id,
            username,
            email,
            status,
        })
    }

    #[tracing::instrument(name = "Fetching a subscription by email in postgres")]
    async fn get_subscription_by_email(
        &self,
        email: &str,
    ) -> Result<Option<Subscription>, SubscriptionError> {
        let saved = sqlx::query!(
            r#"SELECT id, email, username, status::text FROM main.subscriptions WHERE email = $1"#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .context(format!("Could not get subscription for {email}"))?;
        tracing::info!("saved: {saved:?}");
        match saved {
            None => Ok(None),
            Some(rec) => {
                let username = SubscriberName::parse(rec.username).map_err(|err| {
                    SubscriptionError::Validation {
                        context: format!("Invalid username stored in the database: {err}"),
                    }
                })?;
                let email = SubscriberEmail::parse(rec.email).map_err(|err| {
                    SubscriptionError::Validation {
                        context: format!("Invalid email stored in the database: {err}"),
                    }
                })?;
                let status = SubscriptionStatus::from_str(&rec.status.unwrap_or_default())
                    .map_err(|err| SubscriptionError::Validation {
                        context: format!("Invalid status stored in the database: {err}"),
                    })?;
                Ok(Some(Subscription {
                    id: rec.id,
                    username,
                    email,
                    status,
                }))
            }
        }
    }

    #[tracing::instrument(name = "Fetching a subscriber id by token in postgres")]
    async fn get_subscriber_id_by_token(
        &self,
        token: &str,
    ) -> Result<Option<Uuid>, SubscriptionError> {
        let saved = sqlx::query!(
            r#"SELECT subscriber_id FROM main.subscription_tokens WHERE subscription_token = $1"#,
            token
        )
        .fetch_optional(&self.pool)
        .await
        .context(format!("Could not get subscriber id for {token}"))?;
        tracing::info!("saved: {saved:?}");
        Ok(saved.map(|r| r.subscriber_id))
    }

    #[tracing::instrument(name = "Fetching a token using the subscriber's id in postgres")]
    async fn get_token_by_subscriber_id(
        &self,
        id: &Uuid,
    ) -> Result<Option<String>, SubscriptionError> {
        let saved = sqlx::query!(
            r#"SELECT subscription_token FROM main.subscription_tokens WHERE subscriber_id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .context(format!("Could not get token from subscriber id {id}"))?;
        Ok(saved.map(|r| r.subscription_token))
    }

    #[tracing::instrument(name = "Deleting subscription token")]
    async fn delete_confirmation_token(&self, id: &Uuid) -> Result<(), SubscriptionError> {
        sqlx::query!(
            r#"DELETE FROM main.subscription_tokens WHERE subscriber_id = $1"#,
            id
        )
        .execute(&self.pool)
        .await
        .context(format!(
            "Could not delete subscription token for subscriber id {id}"
        ))?;
        Ok(())
    }

    #[tracing::instrument(name = "Confirming subscriber")]
    async fn confirm_subscriber_by_id_and_delete_token(
        &self,
        id: &Uuid,
    ) -> Result<(), SubscriptionError> {
        sqlx::query!(
            r#"UPDATE main.subscriptions SET status = $1 WHERE id = $2"#,
            SubscriptionStatus::Confirmed as SubscriptionStatus,
            id
        )
        .execute(&self.pool)
        .await
        .context(format!("Could not confirm subscriber by id {id}"))?;
        sqlx::query!(
            r#"DELETE FROM main.subscription_tokens WHERE subscriber_id = $1"#,
            id
        )
        .execute(&self.pool)
        .await
        .context(format!(
            "Could not delete subscription token for subscriber id {id}"
        ))?;
        Ok(())
    }

    #[tracing::instrument(name = "Confirming subscriber")]
    async fn get_confirmed_subscribers_email(
        &self,
    ) -> Result<Vec<ConfirmedSubscriber>, SubscriptionError> {
        //Create a fallback password hash to enforce doing the same amount
        //of work whether we have a user account in the db or not.
        let saved = sqlx::query!(
            r#"SELECT email FROM main.subscriptions WHERE status = $1"#,
            SubscriptionStatus::Confirmed as SubscriptionStatus,
        )
        .fetch_all(&self.pool)
        .await
        .context("Could not get a list of confirmed subscribers")?;
        saved
            .into_iter()
            .map(|r| match SubscriberEmail::try_from(r.email) {
                Ok(email) => Ok(ConfirmedSubscriber { email }),
                Err(err) => Err(SubscriptionError::Validation { context: err }),
            })
            .collect()
    }
}

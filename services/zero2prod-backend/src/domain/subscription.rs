use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::SubscriberEmail;
use crate::domain::SubscriberName;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub email: SubscriberEmail,
    pub username: SubscriberName,
    pub status: SubscriptionStatus,
}

#[derive(sqlx::Type, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[sqlx(type_name = "subscription_status")]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    PendingConfirmation,
    Confirmed,
}

impl FromStr for SubscriptionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending_confirmation" => Ok(SubscriptionStatus::PendingConfirmation),
            "confirmed" => Ok(SubscriptionStatus::Confirmed),
            _ => Err(format!("Invalid Subscription Status: {s}")),
        }
    }
}

impl SubscriptionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SubscriptionStatus::Confirmed => "confirmed",
            SubscriptionStatus::PendingConfirmation => "pending_confirmation",
        }
    }
}

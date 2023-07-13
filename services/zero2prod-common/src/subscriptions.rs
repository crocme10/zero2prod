use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// This is what we return to the user in response to the subscription request.
/// Currently this is just a placeholder, and it does not return any useful
/// information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionsResp {
    pub subscription: Subscription,
}

/// This is the information sent by the user to request a subscription.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubscriptionRequest {
    pub username: String,
    pub email: String,
}

/// FIXME This is almost a copy of backend/domain/subscription::Subscription.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

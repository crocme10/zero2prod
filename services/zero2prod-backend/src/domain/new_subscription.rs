use crate::domain::SubscriberEmail;
use crate::domain::SubscriberName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct NewSubscription {
    pub email: SubscriberEmail,
    pub username: SubscriberName,
}

impl TryFrom<SubscriptionRequest> for NewSubscription {
    type Error = String;

    fn try_from(request: SubscriptionRequest) -> Result<Self, Self::Error> {
        let SubscriptionRequest { username, email } = request;

        let username = SubscriberName::try_from(username)?;

        let email = SubscriberEmail::try_from(email)?;

        Ok(NewSubscription { username, email })
    }
}

/// This is the information sent by the user to request a subscription.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubscriptionRequest {
    pub username: String,
    pub email: String,
}

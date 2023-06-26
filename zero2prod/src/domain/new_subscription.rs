use crate::domain::SubscriberEmail;
use crate::domain::SubscriberName;
use crate::routes::subscriptions::SubscriptionRequest;

#[derive(Debug)]
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

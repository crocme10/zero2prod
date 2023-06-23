use crate::domain::SubscriberEmail;
use crate::domain::SubscriberName;

#[derive(Debug)]
pub struct NewSubscription {
    pub email: SubscriberEmail,
    pub username: SubscriberName,
}

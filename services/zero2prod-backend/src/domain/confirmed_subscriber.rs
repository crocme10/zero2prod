use crate::domain::SubscriberEmail;

#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedSubscriber {
    pub email: SubscriberEmail,
}

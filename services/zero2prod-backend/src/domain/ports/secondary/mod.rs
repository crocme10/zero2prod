pub mod authentication_storage;
pub mod subscription_storage;
pub mod email_service;

pub use authentication_storage::{Error as AuthenticationError, AuthenticationStorage};
pub use subscription_storage::{Error as SubscriptionError, SubscriptionStorage};
pub use email_service::{EmailService, Email, Error as EmailError};

#[cfg(test)]
pub use authentication_storage::MockAuthenticationStorage;

#[cfg(test)]
pub use subscription_storage::MockSubscriptionStorage;

#[cfg(test)]
pub use email_service::MockEmailService;


pub mod authentication_storage;
pub mod email_service;
pub mod subscription_storage;

pub use authentication_storage::{AuthenticationStorage, Error as AuthenticationError};
pub use email_service::{Email, EmailService, Error as EmailError};
pub use subscription_storage::{Error as SubscriptionError, SubscriptionStorage};

#[cfg(test)]
pub use authentication_storage::MockAuthenticationStorage;

#[cfg(test)]
pub use subscription_storage::MockSubscriptionStorage;

#[cfg(test)]
pub use email_service::MockEmailService;

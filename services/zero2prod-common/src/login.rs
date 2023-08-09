use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// This is what we return to the user in response to the login request.
/// Currently this is just a placeholder, and it does not return any useful
/// information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResp {
    pub user: User,
}

/// This is the information sent by the user to login
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
}

use base64::engine::general_purpose;
use base64::Engine;
use fake::locales::Data;
use fake::Dummy;
use rand::prelude::SliceRandom;
use secrecy::{ExposeSecret, Secret};

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

impl Credentials {
    pub fn encode(&self) -> String {
        let credentials = format!("{}:{}", self.username, self.password.expose_secret());
        general_purpose::STANDARD.encode(credentials.as_bytes())
    }
}

pub struct C<L>(pub L);

impl<L: Data> Dummy<C<L>> for Credentials {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_config: &C<L>, rng: &mut R) -> Self {
        let username = *L::NAME_FIRST_NAME.choose(rng).unwrap();
        let password = *L::LOREM_WORD.choose(rng).unwrap();
        Credentials {
            username: username.into(),
            password: Secret::new(password.to_string()),
        }
    }
}

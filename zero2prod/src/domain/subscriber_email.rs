use validator::validate_email;

#[derive(Debug, Clone, PartialEq)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid subscriber email.", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<String> for SubscriberEmail {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        SubscriberEmail::parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use speculoos::prelude::*;

    #[test]
    fn empty_string_should_be_rejected() {
        let email = "".to_string();
        assert_that(&SubscriberEmail::parse(email)).is_err();
    }

    #[test]
    fn email_missing_at_symbol_should_be_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_that(&SubscriberEmail::parse(email)).is_err();
    }

    #[test]
    fn email_missing_subject_should_be_rejected() {
        let email = "@domain.com".to_string();
        assert_that(&SubscriberEmail::parse(email)).is_err();
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(_g: &mut quickcheck::Gen) -> Self {
            let email = SafeEmail().fake();
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_should_be_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}

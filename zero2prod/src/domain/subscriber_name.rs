use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct SubscriberName(String);

impl SubscriberName {
    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constraints on subscriber names.
    /// This function is the only way to build an instance of SubscriberName
    /// outside of the domain module
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 256;

        // Iterate over all characters in the input `s` to check if any of them matches
        // one of the characters in the forbidden array.
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name.", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl TryFrom<String> for SubscriberName {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        SubscriberName::parse(value)
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub struct NewSubscription {
    pub email: String,
    pub username: SubscriberName,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use speculoos::prelude::*;

    #[test]
    fn a_256_grapheme_long_name_should_be_valid() {
        let name = "a̐".repeat(256);
        assert_that(&SubscriberName::parse(name)).is_ok();
    }

    #[test]
    fn a_name_longer_than_256_graphemes_should_be_rejected() {
        let name = "a".repeat(257);
        assert_that(&SubscriberName::parse(name)).is_err();
    }

    #[test]
    fn whitespace_only_names_should_be_rejected() {
        let name = " ".to_string();
        assert_that(&SubscriberName::parse(name)).is_err();
    }

    #[test]
    fn empty_string_should_be_rejected() {
        let name = "".to_string();
        assert_that(&SubscriberName::parse(name)).is_err();
    }

    #[test]
    fn names_containing_an_invalid_character_should_be_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_that(&SubscriberName::parse(name)).is_err();
        }
    }

    #[test]
    fn a_valid_name_should_be_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_that(&SubscriberName::parse(name)).is_ok();
    }
}

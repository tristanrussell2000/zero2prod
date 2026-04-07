use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SubscriptionToken(String);

impl SubscriptionToken {
    pub fn parse(s: String) -> Result<Self, String> {
        if s.len() != 25 {
            return Err(format!("{} is not a valid subscription token", s));
        }
        if s.chars().any(|c| !c.is_ascii_alphanumeric()) {
            return Err(format!("{} is not a valid subscription token", s));
        }
        Ok(Self(s))
    }
}

impl AsRef<str> for SubscriptionToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::subscription_token::SubscriptionToken;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_26_character_long_token_is_invalid() {
        let token = "a".repeat(26);
        assert_err!(SubscriptionToken::parse(token));
    }

    #[test]
    fn a_25_character_long_token_is_valid() {
        let token = "a".repeat(25);
        assert_ok!(SubscriptionToken::parse(token));
    }

    #[test]
    fn a_24_character_long_token_is_invalid() {
        let token = "a".repeat(24);
        assert_err!(SubscriptionToken::parse(token));
    }

    #[test]
    fn tokens_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string().repeat(25);
            assert_err!(SubscriptionToken::parse(name));
        }
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = String::from("");
        assert_err!(SubscriptionToken::parse(name));
    }
}

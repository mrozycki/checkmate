use rand_core::{OsRng, RngCore};
use secrecy::{ExposeSecret, Secret, SecretString};
use std::{iter, str::FromStr};

pub const SESSION_TOKEN_LENGTH: usize = 32;

#[derive(Clone, Debug)]
pub struct SessionToken(Secret<[u8; SESSION_TOKEN_LENGTH]>);

impl PartialEq for SessionToken {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("invalid token")]
pub struct InvalidToken;

impl FromStr for SessionToken {
    type Err = InvalidToken;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > SESSION_TOKEN_LENGTH * 2 {
            return Err(InvalidToken);
        }
        Ok(SessionToken(Secret::new(
            hex::decode(
                s.bytes()
                    // Pad with 0s to cover cases when we extend session token length
                    .chain(iter::repeat(b'0'))
                    .take(SESSION_TOKEN_LENGTH * 2)
                    .collect::<Vec<_>>(),
            )
            .map_err(|_| InvalidToken)?
            .try_into()
            .unwrap(),
        )))
    }
}

impl SessionToken {
    pub fn generate_new() -> Self {
        let mut pool = [0u8; SESSION_TOKEN_LENGTH];
        OsRng.fill_bytes(&mut pool);
        Self::from_bytes(pool)
    }

    fn from_bytes(bytes: [u8; SESSION_TOKEN_LENGTH]) -> Self {
        Self(bytes.into())
    }

    pub fn to_database_value(&self) -> Secret<Vec<u8>> {
        self.0.expose_secret().to_vec().into()
    }

    pub(crate) fn to_secret_string(&self) -> SecretString {
        self.0
            .expose_secret()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_from_str() {
        let expected = SessionToken::from_bytes([
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        ]);
        let input_str = "000102030405060708090a0b0c0d0e0f0F0E0D0C0B0A09080706050403020100";

        assert_eq!(expected, SessionToken::from_str(input_str).unwrap());
    }

    #[test]
    fn test_token_from_str_padding() {
        let expected_bytes: [u8; SESSION_TOKEN_LENGTH] = [
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            [0u8; 16],
        ]
        .concat()
        .try_into()
        .unwrap();
        let expected = SessionToken(expected_bytes.into());
        let input_str = "000102030405060708090a0b0c0d0e0f";

        assert_eq!(expected, SessionToken::from_str(input_str).unwrap());
    }

    #[test]
    fn test_token_from_wrong_str() {
        let input_str = "ęśąćż";
        let result = SessionToken::from_str(input_str);
        assert!(result.is_err());
        assert_eq!(InvalidToken, result.err().unwrap());
    }

    #[test]
    fn test_token_fmt() {
        let input = SessionToken::from_bytes([
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 255, 254, 253, 252, 251, 250,
            249, 248, 247, 246, 245, 244, 243, 242, 241, 240,
        ]);
        let expected: String =
            "000102030405060708090a0b0c0d0e0ffffefdfcfbfaf9f8f7f6f5f4f3f2f1f0".to_owned();
        let parsed = input.to_secret_string();

        assert_eq!(&expected, parsed.expose_secret());
    }
}

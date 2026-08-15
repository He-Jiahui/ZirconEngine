use std::str::FromStr;

use uuid::{Uuid, Version};

use super::{HubSessionToken, HubSessionTokenParseError};

impl FromStr for HubSessionToken {
    type Err = HubSessionTokenParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let token = Uuid::parse_str(value)?;
        if token.get_version() != Some(Version::Random) {
            return Err(HubSessionTokenParseError::UnsupportedVersion);
        }
        if value != token.to_string() {
            return Err(HubSessionTokenParseError::NonCanonical);
        }
        Ok(Self(token))
    }
}

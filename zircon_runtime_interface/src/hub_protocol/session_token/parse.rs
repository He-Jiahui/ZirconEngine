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
        // 启动参数、JSON 与邮箱文件名只接受同一种文本形式，拒绝 UUID 的等价拼写。
        if value != token.to_string() {
            return Err(HubSessionTokenParseError::NonCanonical);
        }
        Ok(Self(token))
    }
}

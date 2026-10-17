use std::str::FromStr;

use uuid::{Uuid, Version};

use super::{HubSessionToken, HubSessionTokenParseError};

/// 只接受规范文本形式，避免同一 UUID 出现多种协议字符串。
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

use uuid::Uuid;

use super::HubSessionToken;

impl HubSessionToken {
    /// 生成 UUID v4 会话令牌，用来关联一次 Hub 启动与 Editor 回执。
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for HubSessionToken {
    fn default() -> Self {
        Self::new()
    }
}

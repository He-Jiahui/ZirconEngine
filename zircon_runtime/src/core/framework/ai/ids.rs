use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// 注册行为树后得到的运行时句柄；跨 tick 引用树时使用它，不能把创作态字符串 id 当作句柄。
pub struct AiBehaviorTreeId(pub u64);

impl AiBehaviorTreeId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// 注册黑板布局后得到的句柄；代理 tick 中与行为树引用共同决定键验证和观察者绑定。
pub struct AiBlackboardSchemaId(pub u64);

impl AiBlackboardSchemaId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AiAgentId(pub u64);

impl AiAgentId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

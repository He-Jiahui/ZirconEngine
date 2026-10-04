use crate::scene::LevelSystem;

pub trait RuntimeObject {
    fn object_kind(&self) -> &'static str;
}

/// Level 等运行时系统的轻量身份契约；宿主按系统名定位驱动，不从具体 ECS World 推导生命周期。
pub trait RuntimeSystem: RuntimeObject {
    fn system_name(&self) -> &'static str;
}

impl RuntimeObject for LevelSystem {
    fn object_kind(&self) -> &'static str {
        "system"
    }
}

impl RuntimeSystem for LevelSystem {
    fn system_name(&self) -> &'static str {
        "LevelSystem"
    }
}

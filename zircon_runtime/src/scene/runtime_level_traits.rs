use crate::scene::LevelSystem;

pub trait RuntimeObject {
    fn object_kind(&self) -> &'static str;
}

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

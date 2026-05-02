use std::fmt::Debug;

use super::VirtualGeometryRuntimeState;

pub trait VirtualGeometryRuntimeProvider: Debug + Send + Sync {
    fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState>;
}

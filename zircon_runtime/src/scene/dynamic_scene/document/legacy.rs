use serde::Deserialize;

use crate::scene::World;

#[derive(Deserialize)]
pub(super) struct LegacyProjectDocument {
    pub(super) world: World,
}

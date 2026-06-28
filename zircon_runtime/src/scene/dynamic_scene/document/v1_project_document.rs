use serde::Deserialize;

use crate::scene::World;

#[derive(Deserialize)]
pub(super) struct V1ProjectDocument {
    pub(super) world: World,
}

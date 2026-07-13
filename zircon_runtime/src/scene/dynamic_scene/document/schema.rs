use zircon_runtime_interface::serialization::{
    MigrationChain, MigrationStep, PayloadHeader, SchemaId, VersionedSchema,
};

use crate::scene::dynamic_scene::DynamicScene;

use super::migration::migrate_dynamic_scene_v0_to_v1;

pub(super) const DYNAMIC_SCENE_SCHEMA_VERSION: u32 = 1;

impl VersionedSchema for DynamicScene {
    const SCHEMA: SchemaId = SchemaId::new("zircon.scene.dynamic-scene");
    const VERSION: u32 = DYNAMIC_SCENE_SCHEMA_VERSION;

    fn migrations() -> &'static MigrationChain<Self> {
        &DYNAMIC_SCENE_MIGRATIONS
    }
}

static DYNAMIC_SCENE_MIGRATIONS: MigrationChain<DynamicScene> =
    MigrationChain::new(&[MigrationStep::new(0, migrate_dynamic_scene_v0_to_v1)]);

pub(crate) fn current_dynamic_scene_header() -> PayloadHeader {
    PayloadHeader {
        schema_id: DynamicScene::SCHEMA,
        schema_version: DynamicScene::VERSION,
    }
}

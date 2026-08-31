use zircon_runtime_interface::serialization::{
    MigrationChain, MigrationStep, PayloadHeader, SchemaId, VersionedSchema,
};

use crate::scene::dynamic_scene::DynamicScene;

use super::migration::{
    migrate_dynamic_scene_v0_to_v1, migrate_dynamic_scene_v1_to_v2, migrate_dynamic_scene_v2_to_v3,
};

pub(super) const DYNAMIC_SCENE_SCHEMA_VERSION: u32 = 3;

impl VersionedSchema for DynamicScene {
    const SCHEMA: SchemaId = SchemaId::new("zircon.scene.dynamic-scene");
    const VERSION: u32 = DYNAMIC_SCENE_SCHEMA_VERSION;

    fn migrations() -> &'static MigrationChain<Self> {
        &DYNAMIC_SCENE_MIGRATIONS
    }
}

static DYNAMIC_SCENE_MIGRATIONS: MigrationChain<DynamicScene> = MigrationChain::new(&[
    MigrationStep::new(0, migrate_dynamic_scene_v0_to_v1),
    MigrationStep::new(1, migrate_dynamic_scene_v1_to_v2),
    MigrationStep::new(2, migrate_dynamic_scene_v2_to_v3),
]);

pub(crate) fn current_dynamic_scene_header() -> PayloadHeader {
    PayloadHeader {
        schema_id: DynamicScene::SCHEMA,
        schema_version: DynamicScene::VERSION,
    }
}

use zircon_runtime_interface::serialization::{
    MigrationChain, MigrationStep, SchemaId, VersionedSchema,
};

use super::document::ReflectedJsonDocument;
use super::migration::migrate_reflected_json_v0_to_v1;

pub(super) const REFLECTED_JSON_SCHEMA_VERSION: u32 = 1;

impl VersionedSchema for ReflectedJsonDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.scene.reflected-json");
    const VERSION: u32 = REFLECTED_JSON_SCHEMA_VERSION;

    fn migrations() -> &'static MigrationChain<Self> {
        &REFLECTED_JSON_MIGRATIONS
    }
}

static REFLECTED_JSON_MIGRATIONS: MigrationChain<ReflectedJsonDocument> =
    MigrationChain::new(&[MigrationStep::new(0, migrate_reflected_json_v0_to_v1)]);

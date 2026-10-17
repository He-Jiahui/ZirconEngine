//! 供序列化契约测试共用的版本化载荷与迁移链。
//!
//! `FixtureDocument` 的当前版本为 2；静态迁移链让各测试可分别输入 v0、v1 或当前形状，
//! 并通过 loader 观察迁移、schema 匹配和当前类型解码的边界。

#[path = "binary_contract.rs"]
mod binary_contract;
#[path = "binary_malformed_contract.rs"]
mod binary_malformed_contract;
#[path = "legacy_detection.rs"]
mod legacy_detection;
#[path = "load_contract.rs"]
mod load_contract;
#[path = "malformed_contract.rs"]
mod malformed_contract;
#[path = "migration_contract.rs"]
mod migration_contract;
#[path = "migration_failure_contract.rs"]
mod migration_failure_contract;
#[path = "schema_id_contract.rs"]
mod schema_id_contract;
#[path = "write_contract.rs"]
mod write_contract;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{MigrateError, MigrationChain, MigrationStep, SchemaId, VersionedSchema};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 多个 loader 契约测试共用的拥有型当前载荷形状。
struct FixtureDocument {
    label: String,
    count: u32,
}

impl VersionedSchema for FixtureDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.fixture-document");
    const VERSION: u32 = 2;

    fn migrations() -> &'static MigrationChain<Self> {
        &FIXTURE_MIGRATIONS
    }
}

static FIXTURE_MIGRATIONS: MigrationChain<FixtureDocument> = MigrationChain::new(&[
    MigrationStep::new(0, migrate_fixture_v0_to_v1),
    MigrationStep::new(1, migrate_fixture_v1_to_v2),
]);

/// 输入必须是包含 `name` 的 JSON 对象；非对象或缺少字段时返回 `InvalidPayload`。
/// 成功时移除 `name`、将其值写入 `label`，并把 `count` 设为 1，返回 v1 的 JSON 值。
fn migrate_fixture_v0_to_v1(mut value: Value) -> Result<Value, MigrateError> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| MigrateError::invalid_payload("fixture v0 must be an object"))?;
    let label = object
        .remove("name")
        .ok_or_else(|| MigrateError::invalid_payload("fixture v0 is missing name"))?;
    object.insert("label".to_string(), label);
    object.insert("count".to_string(), Value::from(1));
    Ok(value)
}

/// 输入必须是 JSON 对象，且 `count` 必须可由 `Value::as_u64` 取得；否则返回 `InvalidPayload`。
/// 成功时把 `count` 加一并返回 v2 的 JSON 值，之后由加载器解码为 `FixtureDocument`；加法溢出返回 `InvalidPayload`。
fn migrate_fixture_v1_to_v2(mut value: Value) -> Result<Value, MigrateError> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| MigrateError::invalid_payload("fixture v1 must be an object"))?;
    let count = object
        .get("count")
        .and_then(Value::as_u64)
        .ok_or_else(|| MigrateError::invalid_payload("fixture v1 count must be unsigned"))?;
    let count = count
        .checked_add(1)
        .ok_or_else(|| MigrateError::invalid_payload("fixture v1 count overflow"))?;
    object.insert("count".to_string(), Value::from(count));
    Ok(value)
}

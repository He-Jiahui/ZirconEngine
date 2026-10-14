use serde_json::{Map, Value};

use crate::serialization::{Loaded, MigrateError, MigrationChain, MigrationStep, SchemaId};

use super::{admission::validate_toml_complexity, ProjectManifestSummaryError};

/// 共享项目清单解析器输出的当前 schema 版本；可迁移旧输入会被转换到此版本。
///
pub const PROJECT_MANIFEST_FORMAT_VERSION: u32 = 3;
const PROJECT_MANIFEST_V2_FORMAT_VERSION: u32 = 2;
const PROJECT_MANIFEST_SCHEMA: SchemaId = SchemaId::new("zircon.project-manifest");

// 仅作 MigrationChain 的类型标记；迁移函数实际接收和返回 JSON 值域。
struct ProjectManifestValue;

// 数组下标对应 from_version；MigrationChain 在当前目标版本下校验从 0 到 2 的完整链。
static PROJECT_MANIFEST_MIGRATIONS: MigrationChain<ProjectManifestValue> = MigrationChain::new(&[
    MigrationStep::new(0, migrate_v0_to_v1),
    MigrationStep::new(1, migrate_v1_to_v2),
    MigrationStep::new(2, migrate_v2_to_v3),
]);

// TODO: [CR-PROJECT-0003] 此公开入口未检查 MAX_PROJECT_MANIFEST_BYTES；确认是否应在此收敛字节限额，避免新调用方绕过。
/// Parses TOML into the shared JSON value domain and applies the validated manifest chain.
/// 将经调用方限长的 TOML 转为共享值域，并沿统一版本链迁移到当前格式。
/// `migrated_from` 供预检和 Runtime 判断是否允许旧格式继续进入准入。
///
/// # 使用时机与生命周期
/// 供共享摘要解析及 Runtime 适配器复用 TOML 解析、树复杂度检查和格式升级；成功时 migrated_from 保留旧版来源，供调用方决定是否继续准入。
///
/// # 参数
/// document 是已驻留内存的 TOML 文本。
///
/// # 返回值
/// 返回迁移后的 serde_json::Value 与原始 schema 版本事实；本函数不构造摘要 DTO 或完整 Runtime 清单。
///
/// # 错误
/// TOML 语法、树复杂度、版本字段、未来版本或迁移链失败会返回相应摘要错误。
///
/// # 约束
/// 本公开入口自身不执行 MAX_PROJECT_MANIFEST_BYTES 原始输入预算；直接调用者必须先限长。摘要字符串入口与 Runtime 字符串适配器在调用它之前检查该预算。
///
/// # 参考
/// 参考 [`crate::serialization::MigrationChain::migrate_value`]、[`crate::project::ProjectManifestSummary::parse_toml_str`]；Runtime 路径为 zircon_runtime/src/asset/project/manifest/load.rs。
///
pub fn load_project_manifest_value_from_toml_str(
    document: &str,
) -> Result<Loaded<Value>, ProjectManifestSummaryError> {
    let toml_value = toml::from_str::<toml::Value>(document)
        .map_err(|source| ProjectManifestSummaryError::InvalidToml { source })?;
    validate_toml_complexity(&toml_value)?;
    let value = serde_json::to_value(toml_value)
        .map_err(|source| ProjectManifestSummaryError::InvalidShape { source })?;
    let source_version = source_version(&value)?;
    if source_version > PROJECT_MANIFEST_FORMAT_VERSION {
        return Err(ProjectManifestSummaryError::FutureVersion {
            found: source_version,
            supported: PROJECT_MANIFEST_FORMAT_VERSION,
        });
    }
    let migrated_from =
        (source_version < PROJECT_MANIFEST_FORMAT_VERSION).then_some(source_version);
    let value = PROJECT_MANIFEST_MIGRATIONS.migrate_value(
        &PROJECT_MANIFEST_SCHEMA,
        value,
        source_version,
        PROJECT_MANIFEST_FORMAT_VERSION,
    )?;
    Ok(Loaded {
        value,
        migrated_from,
    })
}

/// 从输入对象读取旧 schema 号；缺省版本沿用 v1 兼容路径，显式值必须是 u32 可表示的无符号整数。
///
fn source_version(value: &Value) -> Result<u32, ProjectManifestSummaryError> {
    let Some(raw) = value.get("format_version") else {
        return Ok(1);
    };
    let version = raw
        .as_u64()
        .and_then(|version| u32::try_from(version).ok())
        .ok_or(ProjectManifestSummaryError::InvalidFormatVersion)?;
    Ok(version)
}

/// 拒绝早于受支持项目清单 schema 的 v0 值，不尝试猜测其字段含义。
///
fn migrate_v0_to_v1(_value: Value) -> Result<Value, MigrateError> {
    Err(MigrateError::invalid_payload(
        "project manifest format_version 0 predates the supported manifest schema",
    ))
}

/// 将 v1 对象补齐 v2 的结构版本及兼容默认项，保留其余原字段。
///
fn migrate_v1_to_v2(mut value: Value) -> Result<Value, MigrateError> {
    let object = manifest_object(&mut value, 1)?;
    object.insert(
        "format_version".to_string(),
        Value::from(PROJECT_MANIFEST_V2_FORMAT_VERSION),
    );
    object
        .entry("engine_version_req".to_string())
        .or_insert(Value::Null);
    object
        .entry("asset_roots".to_string())
        .or_insert_with(|| Value::Array(vec![Value::String("assets".to_string())]));
    object.entry("settings".to_string()).or_insert(Value::Null);
    Ok(value)
}

/// 将已检查为对象的 v2 清单标记为当前 v3；此步不改写其它字段。
///
fn migrate_v2_to_v3(mut value: Value) -> Result<Value, MigrateError> {
    let object = manifest_object(&mut value, PROJECT_MANIFEST_V2_FORMAT_VERSION)?;
    object.insert(
        "format_version".to_string(),
        Value::from(PROJECT_MANIFEST_FORMAT_VERSION),
    );
    Ok(value)
}

/// 确认迁移输入为 TOML 表转换出的 JSON object，并把形状错误归到对应源版本。
///
fn manifest_object(
    value: &mut Value,
    version: u32,
) -> Result<&mut Map<String, Value>, MigrateError> {
    value.as_object_mut().ok_or_else(|| {
        MigrateError::invalid_payload(format!("project manifest v{version} must be a TOML table"))
    })
}

//! 在局部候选中建立目录身份、预算、短名和依赖图不变量，成功后才交给目录发布。
//!
//! 此层检查中立身份与显式依赖；默认值、编辑提示及适配器语义由注册端另行验证。

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::reflect::{ReflectError, ReflectFieldId, ReflectFieldInfo};

use super::{ReflectSchemaCatalogEntry, ReflectSchemaFieldIndex};

/// 整批接纳与增量插入共享的类型数量上限。
pub(super) const MAX_REFLECT_SCHEMA_TYPES: usize = 16_384;
/// 每类型字段上限，也保证字段槽位的 `u32` 转换有界。
const MAX_REFLECT_FIELDS_PER_TYPE: usize = 4_096;
/// 当前名称和别名的字节上限；校验同时要求 ASCII 身份键。
const MAX_REFLECT_FIELD_NAME_BYTES: usize = 128;
/// 单个字段历史别名上限，先于索引建立检查。
const MAX_REFLECT_FIELD_ALIASES_PER_FIELD: usize = 32;
/// 一个类型的别名总量上限，以 checked_add 累计。
const MAX_REFLECT_FIELD_ALIASES_PER_TYPE: usize = 4_096;
/// 单项显式依赖上限，不从字段类型隐式增添边。
const MAX_REFLECT_DEPENDENCIES_PER_TYPE: usize = 256;
/// 全目录显式依赖边上限；批接纳和增量接纳使用同一预算。
pub(super) const MAX_REFLECT_SCHEMA_DEPENDENCIES: usize = 65_536;

/// 已在局部候选中完成所有检查的发布材料；不持有外部目录或实时对象。
///
/// # 参考
/// [`super::ReflectSchemaCatalog::try_new`] 消费各索引，并从空的派生缓存开始。
pub(super) struct AdmittedCatalog {
    /// 按完整类型路径排序的规范化项，是其他投影的唯一元数据来源。
    pub(super) entries: BTreeMap<String, ReflectSchemaCatalogEntry>,
    /// 仅保存当前唯一的短名到完整路径映射。
    pub(super) short_paths: BTreeMap<String, String>,
    /// 冲突短名集合；冲突时不保留任意赢家。
    pub(super) ambiguous_short_paths: BTreeSet<String>,
    /// 每类型按原字段声明顺序建立的 ID/旧名称索引。
    pub(super) field_indexes: BTreeMap<String, ReflectSchemaFieldIndex>,
    /// 全目录 ID 的唯一类型和字段所有者，供增量接纳拒绝跨类型冲突。
    pub(super) field_id_owners: HashMap<ReflectFieldId, (String, String)>,
    /// 已接纳的显式依赖边数，后续插入和移除据此维护预算。
    pub(super) dependency_edge_count: usize,
}

/// 消费整批候选，建立完整、无环且身份唯一的目录发布材料。
///
/// # 参数
/// `entries` 可乱序；每个依赖目标必须同时属于该批次。
///
/// # 返回值
/// 成功返回自有元数据及索引；预算、重复路径/ID 或依赖图错误返回 `ReflectError`。
///
/// # 约束与生命周期
/// 所有变更只发生在本次局部候选中；错误不会向现有目录发布部分结果。
/// 接纳保留字段序列，排序别名与依赖，不验证运行时字段值。
///
/// # 参考
/// [`super::ReflectSchemaCatalog::try_new`]、[`canonicalize_and_validate_entry`]、[`dependency_order`]。
pub(super) fn admit_entries(
    entries: Vec<ReflectSchemaCatalogEntry>,
) -> Result<AdmittedCatalog, ReflectError> {
    if entries.len() > MAX_REFLECT_SCHEMA_TYPES {
        return Err(ReflectError::InvalidRegistration {
            type_path: "<schema-catalog>".to_string(),
            reason: format!(
                "reflection catalog must not contain more than {MAX_REFLECT_SCHEMA_TYPES} types"
            ),
        });
    }

    let mut admitted = BTreeMap::new();
    let mut global_field_ids = HashMap::<ReflectFieldId, (String, String)>::new();
    let mut dependency_count = 0_usize;
    for mut entry in entries {
        let type_path = entry.registration.type_path.type_path().to_string();
        if admitted.contains_key(&type_path) {
            return Err(ReflectError::DuplicateTypePath { type_path });
        }
        canonicalize_and_validate_entry(&mut entry)?;
        dependency_count = dependency_count
            .checked_add(entry.dependencies.len())
            .ok_or_else(|| {
                invalid_registration(&type_path, "schema dependency count overflowed")
            })?;
        if dependency_count > MAX_REFLECT_SCHEMA_DEPENDENCIES {
            return Err(invalid_registration(
                &type_path,
                &format!(
                    "reflection catalog must not contain more than {MAX_REFLECT_SCHEMA_DEPENDENCIES} dependencies"
                ),
            ));
        }
        // 插入全局所有者后若发现冲突，只丢弃本次候选；尚没有任何活动目录被修改。
        for field in &entry.registration.type_info.fields {
            if let Some((owner_type, owner_field)) =
                global_field_ids.insert(field.id, (type_path.clone(), field.name.clone()))
            {
                return Err(invalid_field_registration(
                    &type_path,
                    &field.name,
                    &format!(
                        "field ID `{}` is already owned by `{owner_type}.{owner_field}`",
                        field.id
                    ),
                ));
            }
        }
        admitted.insert(type_path, entry);
    }

    // 收齐整批才检查依赖，允许输入先列消费者；字段索引只在完整图通过后发布。
    let (short_paths, ambiguous_short_paths) = build_short_path_index(&admitted);
    dependency_order(&admitted)?;
    let field_indexes = admitted
        .iter()
        .map(|(type_path, entry)| {
            (
                type_path.clone(),
                ReflectSchemaFieldIndex::from_fields(&entry.registration.type_info.fields),
            )
        })
        .collect();
    Ok(AdmittedCatalog {
        entries: admitted,
        short_paths,
        ambiguous_short_paths,
        field_indexes,
        field_id_owners: global_field_ids,
        dependency_edge_count: dependency_count,
    })
}

// 这里只校验中立目录的身份与依赖约束；值类型、编辑提示等运行时语义由注册端继续校验。
/// 就地检查并规范化一个候选的字段身份与依赖列表。
///
/// # 参数与返回值
/// `entry` 是可丢弃的局部候选；成功保留字段次序、排序别名和依赖，违规返回错误。
///
/// # 约束
/// 不检查目录内的依赖目标、跨类型 ID 或环；调用方必须继续执行整批/增量检查。
/// 失败前可能已排序部分列表，公开只读校验入口会先克隆以保留调用方输入。
///
/// # 参考
/// [`super::ReflectSchemaCatalog::validate_entry`]、`prepare_insert`、[`admit_entries`]。
pub(super) fn canonicalize_and_validate_entry(
    entry: &mut ReflectSchemaCatalogEntry,
) -> Result<(), ReflectError> {
    let type_path = entry.registration.type_path.type_path().to_string();
    if entry.registration.type_info.fields.len() > MAX_REFLECT_FIELDS_PER_TYPE {
        return Err(invalid_registration(
            &type_path,
            &format!("type must not declare more than {MAX_REFLECT_FIELDS_PER_TYPE} fields"),
        ));
    }
    validate_field_identities(&type_path, &mut entry.registration.type_info.fields)?;

    if entry.dependencies.len() > MAX_REFLECT_DEPENDENCIES_PER_TYPE {
        return Err(invalid_registration(
            &type_path,
            &format!(
                "type must not declare more than {MAX_REFLECT_DEPENDENCIES_PER_TYPE} schema dependencies"
            ),
        ));
    }
    for dependency in &entry.dependencies {
        super::super::type_path::validate_type_path(dependency)?;
        if dependency == &type_path {
            return Err(invalid_registration(
                &type_path,
                "schema types must not depend on themselves",
            ));
        }
    }
    // 排序使依赖编码稳定；重复依赖仍是错误，不能靠去重掩盖候选身份问题。
    entry.dependencies.sort_unstable();
    if let Some(duplicate) = entry
        .dependencies
        .windows(2)
        .find(|pair| pair[0] == pair[1])
        .map(|pair| pair[0].clone())
    {
        return Err(invalid_registration(
            &type_path,
            &format!("duplicate schema dependency `{duplicate}`"),
        ));
    }
    Ok(())
}

/// 分两遍检查当前名和别名的唯一性，再排序别名集合。
///
/// # 参数与约束
/// `type_path` 用于错误归属；`fields` 的数量已由调用方限制，字段次序始终保留。
///
/// # 返回值
/// ID、名称、别名或预算冲突返回字段错误；成功后每个导入名称只对应一个字段。
///
/// # 参考
/// [`canonicalize_and_validate_entry`]、[`super::field_index::ReflectSchemaFieldIndex::from_fields`]。
fn validate_field_identities(
    type_path: &str,
    fields: &mut [ReflectFieldInfo],
) -> Result<(), ReflectError> {
    let mut field_ids = HashSet::with_capacity(fields.len());
    let mut field_names = HashSet::with_capacity(fields.len());
    let mut alias_count = 0_usize;
    for field in fields.iter() {
        if !field_ids.insert(field.id) {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                &format!("duplicate reflected field ID `{}`", field.id),
            ));
        }
        validate_field_key(type_path, field, "name", &field.name)?;
        if !field_names.insert(field.name.as_str()) {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                &format!("duplicate reflected field `{}`", field.name),
            ));
        }
        if field.aliases.len() > MAX_REFLECT_FIELD_ALIASES_PER_FIELD {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                &format!(
                    "field must not declare more than {MAX_REFLECT_FIELD_ALIASES_PER_FIELD} aliases"
                ),
            ));
        }
        alias_count = alias_count
            .checked_add(field.aliases.len())
            .ok_or_else(|| {
                invalid_field_registration(type_path, &field.name, "field alias count overflowed")
            })?;
        if alias_count > MAX_REFLECT_FIELD_ALIASES_PER_TYPE {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                &format!(
                    "type must not declare more than {MAX_REFLECT_FIELD_ALIASES_PER_TYPE} field aliases"
                ),
            ));
        }
    }

    // 先收集所有当前名，别名才不会因字段顺序不同而遗漏与后续当前名的冲突。
    for field in fields.iter() {
        for alias in &field.aliases {
            validate_field_key(type_path, field, "alias", alias)?;
            if !field_names.insert(alias.as_str()) {
                return Err(invalid_field_registration(
                    type_path,
                    &field.name,
                    &format!("duplicate reflected field name or alias `{alias}`"),
                ));
            }
        }
    }
    // 集合借用了名称和别名；结束借用后才可就地排序别名，保持字段本身的位置。
    drop(field_names);
    for field in fields {
        field.aliases.sort_unstable();
    }
    Ok(())
}

/// 检查一个身份键的非空、已裁剪、字节上限和 ASCII 字符集合。
///
/// `type_path`、`field` 和 `label` 只用于准确诊断；`value` 不做自动修剪或字符替换。
/// 成功返回单位值，违规由 [`invalid_field_registration`] 保留所属字段。
fn validate_field_key(
    type_path: &str,
    field: &ReflectFieldInfo,
    label: &str,
    value: &str,
) -> Result<(), ReflectError> {
    if value.is_empty() || value.trim() != value {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("field {label} must be non-empty and already trimmed"),
        ));
    }
    if value.len() > MAX_REFLECT_FIELD_NAME_BYTES {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("field {label} must not exceed {MAX_REFLECT_FIELD_NAME_BYTES} UTF-8 bytes"),
        ));
    }
    if !value
        .bytes()
        .all(|byte| byte == b'_' || byte == b'-' || byte.is_ascii_alphanumeric())
    {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("field {label} must use ASCII letters, digits, `_`, or `-`"),
        ));
    }
    Ok(())
}

/// 从完整路径目录建立唯一短名映射和冲突集合。
///
/// `entries` 的完整路径键保持权威；返回的两个集合互斥，冲突短名不会选一个类型。
/// 调用者在 [`super::ReflectSchemaCatalog::resolve_type_path`] 中仍先尝试完整路径。
pub(super) fn build_short_path_index(
    entries: &BTreeMap<String, ReflectSchemaCatalogEntry>,
) -> (BTreeMap<String, String>, BTreeSet<String>) {
    let mut short_paths = BTreeMap::new();
    let mut ambiguous = BTreeSet::new();
    for (type_path, entry) in entries {
        let short = entry.registration.type_path.short_type_path();
        if ambiguous.contains(short) {
            continue;
        }
        match short_paths.get(short) {
            None => {
                short_paths.insert(short.to_string(), type_path.clone());
            }
            Some(existing) if existing == type_path => {}
            Some(_) => {
                // 第二个所有者出现即撤销唯一映射；更多同名项继续保留冲突状态。
                short_paths.remove(short);
                ambiguous.insert(short.to_string());
            }
        }
    }
    (short_paths, ambiguous)
}

/// 以稳定的就绪集合顺序验证依赖闭包并生成依赖先于消费者的拓扑顺序。
///
/// # 参数
/// `entries` 是完整路径目录；显式依赖使用完整路径，短名不参与补全。
///
/// # 返回值
/// 缺失目标或有环返回注册错误；成功返回覆盖所有项的自有路径列表。
///
/// # 参考
/// [`admit_entries`] 在发布前验证图，[`super::ReflectSchemaCatalog::dependency_order`] 缓存成功投影。
pub(super) fn dependency_order(
    entries: &BTreeMap<String, ReflectSchemaCatalogEntry>,
) -> Result<Vec<String>, ReflectError> {
    let mut remaining_dependencies = BTreeMap::<String, usize>::new();
    let mut dependents = BTreeMap::<String, Vec<String>>::new();
    for (type_path, entry) in entries {
        remaining_dependencies.insert(type_path.clone(), entry.dependencies.len());
        for dependency in &entry.dependencies {
            if !entries.contains_key(dependency) {
                return Err(invalid_registration(
                    type_path,
                    &format!("missing schema dependency `{dependency}`"),
                ));
            }
            // 反向边用于依赖就绪后降低消费者计数；先确认目标存在，避免遗漏闭包。
            dependents
                .entry(dependency.clone())
                .or_default()
                .push(type_path.clone());
        }
    }
    for consumers in dependents.values_mut() {
        consumers.sort_unstable();
    }

    let mut ready = remaining_dependencies
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(type_path, _)| type_path.clone())
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(entries.len());
    // 每次从就绪项中取字典序最小者；不同输入顺序不会改变同一图的输出顺序。
    while let Some(type_path) = ready.pop_first() {
        order.push(type_path.clone());
        let Some(consumers) = dependents.get(&type_path) else {
            continue;
        };
        for consumer in consumers {
            let remaining = remaining_dependencies
                .get_mut(consumer)
                .expect("catalog dependency consumer must exist");
            *remaining -= 1;
            if *remaining == 0 {
                ready.insert(consumer.clone());
            }
        }
    }
    if order.len() == entries.len() {
        return Ok(order);
    }
    // 剩余计数非零即可证明存在环；所选诊断路径也可能是被环阻塞的下游消费者。
    let cycle_owner = remaining_dependencies
        .into_iter()
        .find(|(_, count)| *count != 0)
        .map(|(type_path, _)| type_path)
        .unwrap_or_else(|| "<schema-catalog>".to_string());
    Err(invalid_registration(
        &cycle_owner,
        "schema dependency graph contains a cycle",
    ))
}

/// 把目录级原因复制为自有注册错误，使局部候选销毁后诊断仍有效。
fn invalid_registration(type_path: &str, reason: &str) -> ReflectError {
    ReflectError::InvalidRegistration {
        type_path: type_path.to_string(),
        reason: reason.to_string(),
    }
}

/// 把字段级原因连同类型和字段名称复制到错误，不借用待丢弃的候选。
fn invalid_field_registration(type_path: &str, field_name: &str, reason: &str) -> ReflectError {
    ReflectError::InvalidFieldRegistration {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
        reason: reason.to_string(),
    }
}

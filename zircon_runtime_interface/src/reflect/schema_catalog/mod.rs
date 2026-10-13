//! 接纳和维护不含对象适配器的反射模式目录，集中处理路径、稳定字段身份和显式依赖。
//!
//! 整批构造验证完整依赖图；增量插入要求目标已存在，替换先重建候选再发布。
//! 借用查询只适用于当前目录，指纹标识元数据内容；实时对象访问及注册代际由调用方维护。

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::reflect::{ReflectError, ReflectFieldId, ReflectTypeRegistration};

// 私有模块分别负责接纳、候选载体、字段索引和指纹编码；公开入口集中维护跨索引一致性。
#[path = "admission.rs"]
mod admission;
#[path = "entry.rs"]
mod entry;
#[path = "field_index.rs"]
mod field_index;
#[path = "fingerprint.rs"]
mod fingerprint;

use admission::{
    admit_entries, build_short_path_index, canonicalize_and_validate_entry, dependency_order,
    MAX_REFLECT_SCHEMA_DEPENDENCIES, MAX_REFLECT_SCHEMA_TYPES,
};
use field_index::ReflectSchemaFieldIndex;
use fingerprint::fingerprint;

/// 公开候选项，供注册端显式提交元数据和依赖。

pub use entry::ReflectSchemaCatalogEntry;
/// 公开内容指纹及其编码版本，供快照与模式查询按同一契约比较。

pub use fingerprint::{ReflectSchemaFingerprint, REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION};

/// Admitted neutral reflection catalog shared by runtime, tooling, editor, and script hosts.
/// 已接纳的中立反射目录；保留规范化元数据、字段索引和依赖关系供多个宿主共享。
///
/// # 使用时机与生命周期
/// 注册端先提交候选；成功接纳后，目录拥有元数据并为查询返回依附 `&self` 的借用。
/// 更新需要独占借用并使派生缓存失效；字段槽位指向这一版字段序列，不是跨版本永久编号。
///
/// # 约束
/// 本目录验证身份、预算和依赖，不安装对象适配器，也不验证字段默认值与编辑提示的运行时相容性。
/// `TypeRegistry` 在发布前补充值语义检查，并另行维护 `schema_catalog_generation`。
///
/// # 参考
/// [`Self::try_new`]、[`Self::field_slot_by_id`]；消费者 `WorldReflection::list_reflect_types` 与 `field_access`。
#[derive(Clone, Debug)]
pub struct ReflectSchemaCatalog {
    /// 完整路径有序映射，保存接纳后的唯一元数据。
    entries: BTreeMap<String, ReflectSchemaCatalogEntry>,
    /// 当前唯一短名映射；完整路径查询始终优先。
    short_paths: BTreeMap<String, String>,
    /// 短名有多个所有者时记录冲突，并从唯一映射移除。
    ambiguous_short_paths: BTreeSet<String>,
    /// 与各项当前字段序列匹配的稳定 ID/旧名称索引。
    field_indexes: BTreeMap<String, ReflectSchemaFieldIndex>,
    /// 全目录字段 ID 所有者；插入查冲突，移除释放该身份。
    field_id_owners: HashMap<ReflectFieldId, (String, String)>,
    /// 已接纳的依赖边总数，供增量预算检查。
    dependency_edge_count: usize,
    /// 依赖拓扑顺序的惰性自有投影；任何目录变更都会重置。
    dependency_order: OnceLock<Vec<String>>,
    /// 对完整规范化元数据计算的惰性指纹；不代替业务注册代际。
    fingerprint: OnceLock<ReflectSchemaFingerprint>,
}

/// Serializable immutable projection of one admitted catalog generation.
/// 一个已接纳目录代际的可序列化投影，包含供重载时复核的指纹与派生索引。
///
/// # 约束与生命周期
/// 投影拥有元数据，不借用活动目录或实时对象；serde 解码本身不会重新接纳。
/// 外来或改写的投影须经 [`ReflectSchemaCatalog::try_from_snapshot`] 重建并比对。
/// 此载体没有实时会话/VM epoch 标记，不能据此判断某个旧槽位仍可用于当前对象。
///
/// # 参考
/// [`ReflectSchemaCatalog::snapshot`]、[`ReflectSchemaCatalog::try_from_snapshot`]；
/// `VmReflectionRegistrySnapshot::is_current` 是运行时另一个具备当前性检查的快照入口。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReflectSchemaCatalogSnapshot {
    /// 指纹编码与投影比较的算法版本；重载先检查这一值。
    pub algorithm_version: u32,
    /// 全目录内容的期望指纹，重载从 `entries` 重新计算后比对。
    pub fingerprint: ReflectSchemaFingerprint,
    /// 自有候选元数据；重载再次检查身份、预算与依赖闭包。
    pub entries: Vec<ReflectSchemaCatalogEntry>,
    /// 按字典序列出的冲突短名；重载要求与重建结果逐项相等。
    pub ambiguous_short_type_paths: Vec<String>,
    /// 稳定拓扑路径序列；重载比对顺序，不只比较集合。
    pub dependency_order: Vec<String>,
}

impl ReflectSchemaCatalog {
    /// 整批接纳允许输入乱序，但所有依赖必须存在且无环。
    ///
    /// # 参数
    /// `entries` 的所有权移入候选；输入顺序不限，显式依赖必须在整批中存在。
    ///
    /// # 返回值
    /// 成功返回自有目录；重复路径/字段身份、预算或依赖图违规返回错误，不发布部分目录。
    ///
    /// # 参考
    /// [`Self::try_insert`] 的依赖需提前接纳；[`Self::validate_entry`] 不检查整批关系。
    pub fn try_new(entries: Vec<ReflectSchemaCatalogEntry>) -> Result<Self, ReflectError> {
        let admitted = admit_entries(entries)?;
        Ok(Self {
            entries: admitted.entries,
            short_paths: admitted.short_paths,
            ambiguous_short_paths: admitted.ambiguous_short_paths,
            field_indexes: admitted.field_indexes,
            field_id_owners: admitted.field_id_owners,
            dependency_edge_count: admitted.dependency_edge_count,
            dependency_order: OnceLock::new(),
            fingerprint: OnceLock::new(),
        })
    }

    /// 重建目录并比对算法版本、内容指纹及派生投影，拒绝过期或改写的快照。
    ///
    /// # 参数与返回值
    /// `snapshot` 的项重新接纳；版本、内容、冲突短名或拓扑投影不匹配时返回错误。
    ///
    /// # 约束
    /// 这里拒绝的过期是内容/派生投影不一致，不检测实时注册代际，也不校验外来载荷的读取预算。
    ///
    /// # 参考
    /// [`Self::snapshot`]；`catalog_rejects_tampered_or_unknown_version_snapshots`。
    pub fn try_from_snapshot(snapshot: ReflectSchemaCatalogSnapshot) -> Result<Self, ReflectError> {
        if snapshot.algorithm_version != REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION {
            return Err(ReflectError::InvalidRegistration {
                type_path: "<schema-catalog>".to_string(),
                reason: format!(
                    "unsupported reflection schema catalog algorithm version {}; expected {}",
                    snapshot.algorithm_version, REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION
                ),
            });
        }
        // 先重建可信目录，再校验随载荷附带的投影，不能直接把解码后的索引作为权威。
        let expected_fingerprint = snapshot.fingerprint;
        let catalog = Self::try_new(snapshot.entries)?;
        if catalog.fingerprint() != expected_fingerprint {
            return Err(ReflectError::InvalidRegistration {
                type_path: "<schema-catalog>".to_string(),
                reason: format!(
                    "reflection schema catalog fingerprint mismatch: expected {expected_fingerprint}, computed {}",
                    catalog.fingerprint()
                ),
            });
        }
        if snapshot.ambiguous_short_type_paths
            != catalog
                .ambiguous_short_paths
                .iter()
                .cloned()
                .collect::<Vec<_>>()
        {
            return Err(ReflectError::InvalidRegistration {
                type_path: "<schema-catalog>".to_string(),
                reason: "reflection schema catalog ambiguous short-path projection is stale"
                    .to_string(),
            });
        }
        if !snapshot
            .dependency_order
            .iter()
            .map(String::as_str)
            .eq(catalog.dependency_order())
        {
            return Err(ReflectError::InvalidRegistration {
                type_path: "<schema-catalog>".to_string(),
                reason: "reflection schema catalog dependency order projection is stale"
                    .to_string(),
            });
        }
        Ok(catalog)
    }

    /// 返回完整目录的内容指纹，首次计算后缓存，更新目录时失效。
    ///
    /// 返回值是自有可复制的内容标识；过滤后的模式响应仍使用完整目录的指纹。
    /// 参考 [`Self::snapshot`] 与 `WorldReflection::list_reflect_types`。
    pub fn fingerprint(&self) -> ReflectSchemaFingerprint {
        *self.fingerprint.get_or_init(|| fingerprint(&self.entries))
    }

    /// 克隆当前元数据及派生投影，得到不依赖目录借用的快照值。
    ///
    /// # 返回值与生命周期
    /// 返回按完整路径排序的项、冲突短名及稳定拓扑顺序；调用方可独立保存该值。
    /// 重新使用外来快照须经 [`Self::try_from_snapshot`]，此处不附加实时注册代际。
    ///
    /// # 参考
    /// [`ReflectSchemaCatalogSnapshot`]、[`Self::fingerprint`]。
    pub fn snapshot(&self) -> ReflectSchemaCatalogSnapshot {
        ReflectSchemaCatalogSnapshot {
            algorithm_version: REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION,
            fingerprint: self.fingerprint(),
            entries: self.entries.values().cloned().collect(),
            ambiguous_short_type_paths: self.ambiguous_short_paths.iter().cloned().collect(),
            dependency_order: self.dependency_order().map(str::to_string).collect(),
        }
    }

    /// 按完整路径顺序借用目录项；迭代器及其中引用都依附当前目录借用。
    pub fn entries(&self) -> impl Iterator<Item = &ReflectSchemaCatalogEntry> {
        self.entries.values()
    }

    /// 按完整路径顺序借用注册元数据，供模式列表投影；不触发实时对象或适配器访问。
    pub fn registrations(&self) -> impl Iterator<Item = &ReflectTypeRegistration> {
        self.entries.values().map(|entry| &entry.registration)
    }

    /// 先解析完整路径或唯一短名，再借用该类型的权威注册元数据。
    ///
    /// `type_path` 不会选择冲突短名的任意所有者；错误来自 [`Self::resolve_type_path`]。
    /// 返回引用依附目录借用，更新后须重新查询。
    pub fn registration(&self, type_path: &str) -> Result<&ReflectTypeRegistration, ReflectError> {
        let type_path = self.resolve_type_path(type_path)?;
        Ok(&self
            .entries
            .get(type_path)
            .expect("resolved schema type path must have an entry")
            .registration)
    }

    /// 将输入解析为目录拥有的完整类型路径，完整路径命中优先于短名。
    ///
    /// # 参数与返回值
    /// `type_path` 可以是完整路径或唯一短名；未知或冲突短名返回对应错误。
    /// 返回切片借用目录的路径键，不借用输入字符串。
    ///
    /// # 参考
    /// [`Self::registration`]、[`Self::contains`]、[`Self::ambiguous_short_type_paths`]。
    pub fn resolve_type_path(&self, type_path: &str) -> Result<&str, ReflectError> {
        if let Some((canonical, _)) = self.entries.get_key_value(type_path) {
            return Ok(canonical.as_str());
        }
        if let Some(canonical) = self.short_paths.get(type_path) {
            return Ok(canonical.as_str());
        }
        if self.ambiguous_short_paths.contains(type_path) {
            return Err(ReflectError::AmbiguousShortTypePath {
                short_type_path: type_path.to_string(),
            });
        }
        Err(ReflectError::UnknownType {
            type_path: type_path.to_string(),
        })
    }

    /// 将稳定字段 ID 解析为当前注册字段顺序中的槽位，供对象适配器读取或写入。
    ///
    /// # 参数与返回值
    /// `type_path` 先按完整路径/唯一短名解析；`field_id` 必须属于该类型。
    /// 返回原字段声明序列的 `u32` 槽位，未知身份返回字段错误，不按名称推断。
    ///
    /// # 生命周期与参考
    /// 槽位只对当前注册序列有效；`field_access` 进一步核对该位置的 ID，随后才调用对象适配器。
    /// [`Self::resolve_legacy_field_id`] 仅供显式旧名称导入。
    pub fn field_slot_by_id(
        &self,
        type_path: &str,
        field_id: ReflectFieldId,
    ) -> Result<u32, ReflectError> {
        let type_path = self.resolve_type_path(type_path)?;
        self.field_indexes[type_path]
            .field_slot(field_id)
            .ok_or_else(|| ReflectError::UnknownField {
                type_path: type_path.to_string(),
                field_name: field_id.to_string(),
            })
    }

    /// Resolves current or historical field names for one explicitly scoped legacy import.
    /// 仅在明确指定的类型内，把当前名称或历史别名解析为稳定 ID。
    ///
    /// # 参数与返回值
    /// `type_path` 先解析，`field_name` 精确匹配接纳后的名称索引；未知名称返回字段错误。
    ///
    /// # 约束与参考
    /// 不跨类型搜索，也不会在按 ID 访问失败时自动回退；参见 [`Self::field_slot_by_id`]。
    pub fn resolve_legacy_field_id(
        &self,
        type_path: &str,
        field_name: &str,
    ) -> Result<ReflectFieldId, ReflectError> {
        let type_path = self.resolve_type_path(type_path)?;
        self.field_indexes[type_path]
            .legacy_field_id(field_name)
            .ok_or_else(|| ReflectError::UnknownField {
                type_path: type_path.to_string(),
                field_name: field_name.to_string(),
            })
    }

    /// 借用依赖先于消费者的稳定拓扑序列，首次查询缓存该投影。
    ///
    /// 同时就绪的项按完整路径字典序选取；借用依附本目录，变更后须重新取得。
    /// [`Self::try_new`] 已证明依赖闭包无环，故内部缓存构造的 `expect` 依赖接纳不变量。
    pub fn dependency_order(&self) -> impl Iterator<Item = &str> {
        self.dependency_order
            .get_or_init(|| {
                dependency_order(&self.entries)
                    .expect("an admitted schema catalog has an acyclic dependency graph")
            })
            .iter()
            .map(String::as_str)
    }

    /// 按字典序借用当前冲突短名，供诊断和快照投影；删除或替换后集合可能改变。
    pub fn ambiguous_short_type_paths(&self) -> impl Iterator<Item = &str> {
        self.ambiguous_short_paths.iter().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 检查完整路径或唯一短名是否存在；仅存在于冲突集合的短名返回假。
    ///
    /// 直接索引查询不构造解析错误；参考 [`Self::resolve_type_path`] 和 [`Self::contains_type_path`]。
    pub fn contains(&self, type_path: &str) -> bool {
        self.entries.contains_key(type_path) || self.short_paths.contains_key(type_path)
    }

    /// 只检查完整路径键，不把短名解析为类型；注册端用它避免混淆候选身份。
    pub fn contains_type_path(&self, type_path: &str) -> bool {
        self.entries.contains_key(type_path)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 克隆并检查单项中立身份/依赖形状，保留调用方的候选原值。
    ///
    /// # 参数与返回值
    /// `entry` 不必已属于目录；单项预算或身份违规返回错误，成功只证明这一项的局部约束。
    ///
    /// # 约束与参考
    /// 不证明依赖目标存在、全目录 ID 唯一或图无环；需继续 [`Self::validate_insert`] 或 [`Self::try_new`]。
    /// 注册端 `validate_registration` 还检查默认值和编辑元数据。
    pub fn validate_entry(entry: &ReflectSchemaCatalogEntry) -> Result<(), ReflectError> {
        let mut entry = entry.clone();
        canonicalize_and_validate_entry(&mut entry)
    }

    /// 在当前目录上预检一次增量候选，不发布或改变派生缓存。
    ///
    /// `entry` 先克隆；返回结果与本目录当前状态下的 [`Self::try_insert`] 接纳检查对应。
    /// 预检不预留身份，后来更改目录后必须重新校验。
    pub fn validate_insert(&self, entry: &ReflectSchemaCatalogEntry) -> Result<(), ReflectError> {
        self.prepare_insert(entry.clone()).map(|_| ())
    }

    /// 增量接纳要求依赖已入目录；成功后刷新短名、字段索引和派生缓存。
    ///
    /// # 参数与返回值
    /// `entry` 是新完整路径项；重复类型/ID、预算或缺失依赖返回错误。
    /// 所有可返回的校验错误发生在发布前，原目录保持原值；成功接纳规范化项及其索引。
    ///
    /// # 约束与参考
    /// 依赖只能指向已存在项且不得自依赖，因此插入不会给既有无环图引入回边。
    /// [`Self::try_new`] 支持整批乱序依赖；[`Self::validate_insert`] 仅预检。
    pub fn try_insert(&mut self, entry: ReflectSchemaCatalogEntry) -> Result<(), ReflectError> {
        // 先完成所有可失败校验，再同步元数据、身份索引、短名和缓存，避免暴露部分接纳。
        let (type_path, entry, dependency_edge_count) = self.prepare_insert(entry)?;
        let short_path = entry.registration.type_path.short_type_path().to_string();
        for field in &entry.registration.type_info.fields {
            self.field_id_owners
                .insert(field.id, (type_path.clone(), field.name.clone()));
        }
        self.field_indexes.insert(
            type_path.clone(),
            ReflectSchemaFieldIndex::from_fields(&entry.registration.type_info.fields),
        );
        self.entries.insert(type_path.clone(), entry);
        self.dependency_edge_count = dependency_edge_count;
        self.update_short_path_index(&type_path, &short_path);
        self.invalidate_derived_products();
        Ok(())
    }

    /// 在只读目录上规范化自有候选并完成增量检查，返回发布所需完整路径和新边数。
    ///
    /// 失败只丢弃局部候选，不修改目录；依赖存在与全局 ID 唯一在发布前证明。
    /// 参考 [`Self::validate_insert`]、[`Self::try_insert`]。
    fn prepare_insert(
        &self,
        mut entry: ReflectSchemaCatalogEntry,
    ) -> Result<(String, ReflectSchemaCatalogEntry, usize), ReflectError> {
        let type_path = entry.registration.type_path.type_path().to_string();
        if self.entries.contains_key(&type_path) {
            return Err(ReflectError::DuplicateTypePath { type_path });
        }
        if self.entries.len() >= MAX_REFLECT_SCHEMA_TYPES {
            return Err(ReflectError::InvalidRegistration {
                type_path,
                reason: format!(
                    "reflection catalog must not contain more than {MAX_REFLECT_SCHEMA_TYPES} types"
                ),
            });
        }
        canonicalize_and_validate_entry(&mut entry)?;
        let dependency_edge_count = self
            .dependency_edge_count
            .checked_add(entry.dependencies.len())
            .ok_or_else(|| ReflectError::InvalidRegistration {
                type_path: type_path.clone(),
                reason: "schema dependency count overflowed".to_string(),
            })?;
        if dependency_edge_count > MAX_REFLECT_SCHEMA_DEPENDENCIES {
            return Err(ReflectError::InvalidRegistration {
                type_path: type_path.clone(),
                reason: format!(
                    "reflection catalog must not contain more than {MAX_REFLECT_SCHEMA_DEPENDENCIES} dependencies"
                ),
            });
        }
        // 完整路径依赖必须已接纳；新项不能作为既有项的依赖，局部插入无需重跑全图排序。
        for dependency in &entry.dependencies {
            if !self.entries.contains_key(dependency) {
                return Err(ReflectError::InvalidRegistration {
                    type_path: type_path.clone(),
                    reason: format!("missing schema dependency `{dependency}`"),
                });
            }
        }
        for field in &entry.registration.type_info.fields {
            if let Some((owner_type, owner_field)) = self.field_id_owners.get(&field.id) {
                return Err(ReflectError::InvalidFieldRegistration {
                    type_path: type_path.clone(),
                    field_name: field.name.clone(),
                    reason: format!(
                        "field ID `{}` is already owned by `{owner_type}.{owner_field}`",
                        field.id
                    ),
                });
            }
        }

        Ok((type_path, entry, dependency_edge_count))
    }

    /// 用同一完整路径的新项替换旧项，先整批重建再原子发布目录值。
    ///
    /// # 参数与返回值
    /// `entry` 的完整路径必须已存在；短名不能代替替换身份。
    /// 重建失败返回错误并保留旧元数据、索引与缓存；成功刷新全部投影。
    ///
    /// # 参考
    /// [`Self::try_new`] 重新验证依赖与全局 ID；`TypeRegistry::upsert_vm_type` 在成功后才替换适配器。
    pub fn try_replace(&mut self, entry: ReflectSchemaCatalogEntry) -> Result<(), ReflectError> {
        let type_path = entry.registration.type_path.type_path().to_string();
        if !self.entries.contains_key(&type_path) {
            return Err(ReflectError::UnknownType { type_path });
        }
        let entries = self
            .entries
            .values()
            .filter(|candidate| candidate.registration.type_path.type_path() != type_path.as_str())
            .cloned()
            .chain(std::iter::once(entry))
            .collect();
        // 右侧先完成接纳，只有 Result 成功才赋值；错误不会把旧项先移出活动目录。
        *self = Self::try_new(entries)?;
        Ok(())
    }

    /// 仅当没有其他目录项依赖该类型时才可移除；未知类型路径返回 None，成功后会清理字段索引并使派生指纹与依赖顺序缓存失效。
    ///
    /// # 参数与返回值
    /// `type_path` 可为完整路径或唯一短名；未知返回空值，冲突短名或仍有直接依赖者返回错误。
    /// 成功返回移出的自有项，原有借用不能跨越独占修改。
    ///
    /// # 约束与参考
    /// 在完整依赖闭包中，拒绝任何直接消费者即可保留其余图的闭包；短名索引重建可消除旧冲突。
    /// [`Self::try_replace`]、`catalog_replace_is_atomic_and_remove_rejects_live_dependents`。
    pub fn try_remove(
        &mut self,
        type_path: &str,
    ) -> Result<Option<ReflectSchemaCatalogEntry>, ReflectError> {
        let canonical = match self.resolve_type_path(type_path) {
            Ok(type_path) => type_path.to_string(),
            Err(ReflectError::UnknownType { .. }) => return Ok(None),
            Err(error) => return Err(error),
        };
        if let Some(dependent) = self.entries.values().find(|entry| {
            entry
                .dependencies
                .iter()
                .any(|dependency| dependency == &canonical)
        }) {
            return Err(ReflectError::InvalidRegistration {
                type_path: canonical,
                reason: format!(
                    "schema type is still required by `{}`",
                    dependent.registration.type_path.type_path()
                ),
            });
        }
        // 拒绝分支均在移除前；成功同时释放全局 ID、边预算、短名投影和惰性缓存。
        let removed = self
            .entries
            .remove(&canonical)
            .expect("resolved schema type path must have an entry");
        self.field_indexes.remove(&canonical);
        self.dependency_edge_count -= removed.dependencies.len();
        for field in &removed.registration.type_info.fields {
            self.field_id_owners.remove(&field.id);
        }
        (self.short_paths, self.ambiguous_short_paths) = build_short_path_index(&self.entries);
        self.invalidate_derived_products();
        Ok(Some(removed))
    }

    /// 清空元数据、身份索引和派生缓存；业务适配器及代际必须由注册拥有者同步清理。
    ///
    /// 参考 `TypeRegistry::clear`；此中立目录本身不持有实时对象。
    pub fn clear(&mut self) {
        self.entries.clear();
        self.short_paths.clear();
        self.ambiguous_short_paths.clear();
        self.field_indexes.clear();
        self.field_id_owners.clear();
        self.dependency_edge_count = 0;
        self.invalidate_derived_products();
    }

    /// 把新项短名投影为唯一映射或冲突状态，不让后续插入覆盖某个任意赢家。
    fn update_short_path_index(&mut self, type_path: &str, short_path: &str) {
        if self.ambiguous_short_paths.contains(short_path) {
            return;
        }
        match self.short_paths.get(short_path) {
            None => {
                self.short_paths
                    .insert(short_path.to_string(), type_path.to_string());
            }
            Some(existing) if existing == type_path => {}
            Some(_) => {
                self.short_paths.remove(short_path);
                self.ambiguous_short_paths.insert(short_path.to_string());
            }
        }
    }

    /// 在已完成目录更新后重置两项惰性产物，下一次查询从当前元数据重新计算。
    fn invalidate_derived_products(&mut self) {
        self.dependency_order = OnceLock::new();
        self.fingerprint = OnceLock::new();
    }
}

// 目录相等只比较权威规范化项；查询是否填充缓存，不应改变目录的值相等性。
impl PartialEq for ReflectSchemaCatalog {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

// 空目录满足预算、身份和依赖图约束，与整批空输入使用同一接纳入口。
impl Default for ReflectSchemaCatalog {
    fn default() -> Self {
        Self::try_new(Vec::new()).expect("an empty reflection schema catalog is valid")
    }
}

// 测试模块只在测试配置中参与编译，覆盖排序稳定性、身份冲突和返回错误时的保留行为。
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

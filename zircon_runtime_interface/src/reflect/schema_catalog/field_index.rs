//! 为一个已接纳类型投影稳定字段 ID 和限定类型的旧名称索引。
//!
//! ID 索引中的槽位始终指向原字段序列；选择排序表或哈希表只改变查找方式。

use std::collections::HashMap;

use crate::reflect::{ReflectFieldId, ReflectFieldInfo};

/// 至多此数量的字段使用排序表；更大的已接纳字段集使用哈希表，槽位语义相同。
const HASHED_FIELD_INDEX_THRESHOLD: usize = 512;

/// 单个类型的派生索引，随目录注册替换而重建；不借用字段元数据。
///
/// # 约束
/// 仅对已验证的字段身份与名称建立索引，重复检测和字段数上限由目录接纳负责。
///
/// # 参考
/// [`super::admission::admit_entries`]、[`super::ReflectSchemaCatalog::try_insert`]。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct ReflectSchemaFieldIndex {
    /// 稳定 ID 到原声明槽位的映射，不使用排序后位置作为槽位。
    stable: StableFieldIndex,
    /// 当前名称和历史别名到 ID 的映射，调用者先选择所属类型。
    legacy_names: HashMap<String, ReflectFieldId>,
}

/// 用同一 `(ID, 原槽位)` 关系支持两种存储；ID 唯一性由接纳入口保证。
#[derive(Clone, Debug, PartialEq)]
enum StableFieldIndex {
    /// 按 ID 排序以支持二分；元组第二项仍是原字段序列的位置。
    Sorted(Vec<(ReflectFieldId, u32)>),
    /// 大字段集直接按 ID 查找原槽位。
    Hashed(HashMap<ReflectFieldId, u32>),
}

// 稳定 ID 只映射到当前字段序列的槽位；旧名称和别名另存为限定在所属类型内的导入索引。
impl ReflectSchemaFieldIndex {
    /// 从已接纳的字段描述复制两种索引，不重排元数据字段。
    ///
    /// # 参数与约束
    /// `fields` 须已通过身份、名称/别名唯一性及 4096 字段上限检查，因而槽位可表示为 `u32`。
    ///
    /// # 返回值与生命周期
    /// 返回自有索引；不保留 `fields` 的借用，元数据替换后应重建。
    ///
    /// # 参考
    /// [`super::ReflectSchemaCatalog::field_slot_by_id`]、`resolve_legacy_field_id`。
    pub(super) fn from_fields(fields: &[ReflectFieldInfo]) -> Self {
        let stable = if fields.len() <= HASHED_FIELD_INDEX_THRESHOLD {
            let mut entries = fields
                .iter()
                .enumerate()
                .map(|(slot, field)| (field.id, slot as u32))
                .collect::<Vec<_>>();
            // 只对查找副本排序；把原 enumerate 槽位保留下来，才能与对象适配器的分派一致。
            entries.sort_unstable_by_key(|(field_id, _)| *field_id);
            StableFieldIndex::Sorted(entries)
        } else {
            StableFieldIndex::Hashed(
                fields
                    .iter()
                    .enumerate()
                    .map(|(slot, field)| (field.id, slot as u32))
                    .collect(),
            )
        };
        // ID 与旧名称索引分开，显式旧导入不会影响日常按稳定 ID 的访问。
        let mut legacy_names = HashMap::with_capacity(
            fields.len()
                + fields
                    .iter()
                    .map(|field| field.aliases.len())
                    .sum::<usize>(),
        );
        for field in fields {
            legacy_names.insert(field.name.clone(), field.id);
            for alias in &field.aliases {
                legacy_names.insert(alias.clone(), field.id);
            }
        }
        Self {
            stable,
            legacy_names,
        }
    }

    /// 返回该 ID 在建索引时的字段槽位；不存在时返回空值，不猜测名称或偏移。
    pub(super) fn field_slot(&self, field_id: ReflectFieldId) -> Option<u32> {
        match &self.stable {
            StableFieldIndex::Sorted(entries) => entries
                .binary_search_by_key(&field_id, |(candidate, _)| *candidate)
                .ok()
                .map(|index| entries[index].1),
            StableFieldIndex::Hashed(entries) => entries.get(&field_id).copied(),
        }
    }

    /// 精确解析此类型内的当前名称或历史别名；不跨类型查找，也不自动调用旧导入。
    pub(super) fn legacy_field_id(&self, field_name: &str) -> Option<ReflectFieldId> {
        self.legacy_names.get(field_name).copied()
    }
}

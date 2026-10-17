//! 绑定值树的只读准入与一次性累计状态。
//!
//! 每次公开校验都从新计数器和根深度一开始；递归子树共享节点与字符串字节累计值。
//! 首个错误立即返回，先前累计状态只随该次调用释放，不修改拥有值树。
//! 这里限制已经构造好的树，不限制解析或反序列化此前发生的分配。

use super::{
    types::validate_identity, UiBindingMapKey, UiBindingValue, UiBindingValueBudget,
    UiBindingValueIdentityKind, UiBindingValueValidationError,
};

impl UiBindingValue {
    /// 按标准上限校验当前完整根值，返回首个领域或预算错误。
    /// 只读借用结束后值保持原样；成功不代表外部资产、实体或提供方当前存在。
    pub fn validate(&self) -> Result<(), UiBindingValueValidationError> {
        self.validate_with_budget(UiBindingValueBudget::STANDARD)
    }

    /// 以整棵值树为单位检查资源上限、身份和数值有效性，供解析入口与提交方做准入。
    /// 自定义预算可收紧限制；只校验某个子值不能证明包含它的完整值树合规。
    ///
    /// # 参数与返回
    /// `budget` 按值提供本次可调上限，不与前次调用共享计数；固定身份和窗口上限仍生效。
    /// 成功仅证明从此根开始的整树符合这些检查，错误时不返回部分通过的子树。
    ///
    /// # 计数与生命周期
    /// 根深度为一；映射键另计节点，字符串键及相关身份计入累计字节。
    /// 校验期间只读借用树，临时累计状态在返回时释放；调用方放宽预算时不会被默认值再次截断。
    pub fn validate_with_budget(
        &self,
        budget: UiBindingValueBudget,
    ) -> Result<(), UiBindingValueValidationError> {
        let mut state = UiBindingValueValidationState::default();
        self.validate_at_depth(budget, &mut state, 1)
    }

    /// 以共享状态检查一个子树：先检查深度，再计当前节点，随后按变体检查自身和子值。
    /// 容器项数先于遍历检查；错误直接向上传播，部分累计不回滚但不会对外保留。
    fn validate_at_depth(
        &self,
        budget: UiBindingValueBudget,
        state: &mut UiBindingValueValidationState,
        depth: usize,
    ) -> Result<(), UiBindingValueValidationError> {
        if depth > budget.max_depth {
            return Err(UiBindingValueValidationError::DepthExceeded {
                actual: depth,
                maximum: budget.max_depth,
            });
        }
        state.add_node(budget)?;
        match self {
            Self::String(value) => state.add_string(budget, value),
            Self::Float(value) if !value.is_finite() => {
                Err(UiBindingValueValidationError::NonFiniteFloat)
            }
            Self::Array(values) => {
                validate_collection_len(values.len(), budget)?;
                for value in values {
                    value.validate_at_depth(budget, state, depth + 1)?;
                }
                Ok(())
            }
            Self::Record(fields) => {
                validate_collection_len(fields.len(), budget)?;
                for (field, value) in fields {
                    validate_identity(UiBindingValueIdentityKind::RecordField, field)?;
                    state.add_string(budget, field)?;
                    value.validate_at_depth(budget, state, depth + 1)?;
                }
                Ok(())
            }
            Self::Map(values) => {
                validate_collection_len(values.len(), budget)?;
                for (key, value) in values.iter() {
                    state.add_node(budget)?;
                    if let UiBindingMapKey::String(key) = key {
                        state.add_string(budget, key)?;
                    }
                    value.validate_at_depth(budget, state, depth + 1)?;
                }
                Ok(())
            }
            Self::Enum(value) => {
                validate_identity(UiBindingValueIdentityKind::EnumType, value.type_id())?;
                validate_identity(UiBindingValueIdentityKind::EnumVariant, value.variant())?;
                state.add_string(budget, value.type_id())?;
                state.add_string(budget, value.variant())?;
                if let Some(payload) = value.payload() {
                    payload.validate_at_depth(budget, state, depth + 1)?;
                }
                Ok(())
            }
            Self::Asset(value) => {
                validate_identity(UiBindingValueIdentityKind::AssetLocator, value.locator())?;
                state.add_string(budget, value.locator())
            }
            Self::Entity(value) => {
                if value.generation() == 0 {
                    Err(UiBindingValueValidationError::ZeroGeneration {
                        kind: UiBindingValueIdentityKind::EntityGeneration,
                    })
                } else {
                    Ok(())
                }
            }
            Self::Optional(value) => {
                if let Some(value) = value.as_deref() {
                    value.validate_at_depth(budget, state, depth + 1)?;
                }
                Ok(())
            }
            Self::CollectionView(value) => {
                value.validate_window()?;
                state.add_string(budget, value.provider().id.as_str())?;
                state.add_string(budget, value.item_schema().id.as_str())
            }
            Self::Unsigned(_) | Self::Signed(_) | Self::Float(_) | Self::Bool(_) | Self::Null => {
                Ok(())
            }
        }
    }
}

/// 仅供一次根值校验持有的累计计数，默认从零开始；不缓存在值对象上。
#[derive(Default)]
struct UiBindingValueValidationState {
    /// 已访问的值节点与映射键数量；使用饱和累计。
    nodes: usize,
    /// 已访问字符串及相应身份的 UTF-8 总字节数；使用饱和累计。
    string_bytes: usize,
}

impl UiBindingValueValidationState {
    /// 将一个节点计入本次状态，再比较调用方上限；失败时该局部状态已更新。
    fn add_node(
        &mut self,
        budget: UiBindingValueBudget,
    ) -> Result<(), UiBindingValueValidationError> {
        self.nodes = self.nodes.saturating_add(1);
        if self.nodes > budget.max_nodes {
            Err(UiBindingValueValidationError::NodeBudgetExceeded {
                actual: self.nodes,
                maximum: budget.max_nodes,
            })
        } else {
            Ok(())
        }
    }

    /// 按借用文本的 UTF-8 长度增加累计字节，再比较上限，不复制字符串。
    fn add_string(
        &mut self,
        budget: UiBindingValueBudget,
        value: &str,
    ) -> Result<(), UiBindingValueValidationError> {
        self.string_bytes = self.string_bytes.saturating_add(value.len());
        if self.string_bytes > budget.max_string_bytes {
            Err(UiBindingValueValidationError::StringBudgetExceeded {
                actual: self.string_bytes,
                maximum: budget.max_string_bytes,
            })
        } else {
            Ok(())
        }
    }
}

/// 比较单个容器的条目数与本次上限；不累计其他容器，也不遍历子值。
fn validate_collection_len(
    actual: usize,
    budget: UiBindingValueBudget,
) -> Result<(), UiBindingValueValidationError> {
    if actual > budget.max_collection_entries {
        Err(UiBindingValueValidationError::CollectionEntriesExceeded {
            actual,
            maximum: budget.max_collection_entries,
        })
    } else {
        Ok(())
    }
}

//! 注册前检查组件声明的结构、默认值和能力一致性；编写时的声明错误在这里形成可定位的组件/字段诊断。

use std::collections::HashSet;

use thiserror::Error;

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiHostCapability, UiPropSchema, UiRenderCapability, UiValue, UiValueKind,
};

/// 描述符准入失败的稳定分类；保留组件及字段身份供项目注册或编写工具定位声明来源。
/// 组件、字段和槽位身份复制为拥有值；诊断可在本次声明借用结束后继续保存。
#[derive(Clone, Debug, Error, PartialEq)]
pub enum UiComponentDescriptorError {
    #[error("component descriptor id must not be empty")]
    EmptyId,
    #[error("component descriptor `{component_id}` display name must not be empty")]
    EmptyDisplayName { component_id: String },
    #[error("component descriptor `{component_id}` role must not be empty")]
    EmptyRole { component_id: String },
    /// 属性/状态名为空或重复，或同一模式内选项 ID 重复；schema_kind 区分诊断类别。
    #[error("component descriptor `{component_id}` has duplicate {schema_kind} schema `{name}`")]
    DuplicateSchemaName {
        component_id: String,
        schema_kind: &'static str,
        name: String,
    },
    #[error("component descriptor `{component_id}` has duplicate slot schema `{name}`")]
    DuplicateSlotName { component_id: String, name: String },
    #[error("component descriptor `{component_id}` default prop `{name}` has no prop schema")]
    MissingDefaultPropSchema { component_id: String, name: String },
    #[error("component descriptor `{component_id}` schema `{name}` default uses {actual:?}, expected {expected:?}")]
    DefaultValueKindMismatch {
        component_id: String,
        name: String,
        expected: UiValueKind,
        actual: UiValueKind,
    },
    /// 任一已提供边界含非有限数，或同时提供的上下界倒置。
    #[error("component descriptor `{component_id}` schema `{name}` has invalid range")]
    InvalidRange { component_id: String, name: String },
    #[error("component descriptor `{component_id}` schema `{name}` has invalid step")]
    InvalidStep { component_id: String, name: String },
    /// 默认值的标量、向量或嵌套数组/映射中出现非有限浮点数。
    #[error("component descriptor `{component_id}` numeric value `{name}` must be finite")]
    NonFiniteNumber { component_id: String, name: String },
    #[error(
        "component descriptor `{component_id}` palette metadata is missing a default node template"
    )]
    MissingPaletteDefaultNode { component_id: String },
    #[error("component descriptor `{component_id}` palette sort key must not be empty")]
    EmptyPaletteSortKey { component_id: String },
    #[error(
        "component descriptor `{component_id}` default node slot `{slot_name}` is not declared"
    )]
    UnknownDefaultNodeSlot {
        component_id: String,
        slot_name: String,
    },
    #[error("component descriptor `{component_id}` virtualized render capability requires virtualized host capability")]
    MissingVirtualizedHostCapability { component_id: String },
}

/// 注册表写入前调用；检查声明自洽性。宿主/渲染端是否支持这些能力和实例输入是否合法仍需各自消费方确认。
///
/// # 参数
/// - `descriptor`：本次借用的完整声明；检查不修改它，也不保存对它的引用。
///
/// # 返回与错误
/// 成功返回 `Ok(())`；失败按身份、属性/状态名称、槽位、模式默认值、默认属性、
/// 调色板和能力的检查次序返回首个错误，不汇总所有诊断。
///
/// # 参考
/// [`crate::ui::component::UiComponentDescriptorRegistry::try_register`] 在写入目录前调用本检查。
pub fn validate_component_descriptor(
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentDescriptorError> {
    if descriptor.id.trim().is_empty() {
        return Err(UiComponentDescriptorError::EmptyId);
    }
    if descriptor.display_name.trim().is_empty() {
        return Err(UiComponentDescriptorError::EmptyDisplayName {
            component_id: descriptor.id.clone(),
        });
    }
    if descriptor.role.trim().is_empty() {
        return Err(UiComponentDescriptorError::EmptyRole {
            component_id: descriptor.id.clone(),
        });
    }

    validate_schema_names(descriptor, "prop", &descriptor.prop_schema)?;
    validate_schema_names(descriptor, "state", &descriptor.state_schema)?;
    validate_slot_names(descriptor)?;
    validate_schema_defaults(descriptor)?;
    validate_default_props(descriptor)?;
    validate_palette(descriptor)?;
    validate_capabilities(descriptor)?;
    Ok(())
}

// 按声明顺序返回第一个重复身份，保留诊断可重复性；成员表的迭代次序不参与错误选择。
fn validate_schema_names(
    descriptor: &UiComponentDescriptor,
    schema_kind: &'static str,
    schemas: &[UiPropSchema],
) -> Result<(), UiComponentDescriptorError> {
    let mut names = HashSet::with_capacity(schemas.len());
    for schema in schemas {
        if schema.name.trim().is_empty() || !names.insert(schema.name.as_str()) {
            return Err(UiComponentDescriptorError::DuplicateSchemaName {
                component_id: descriptor.id.clone(),
                schema_kind,
                name: schema.name.clone(),
            });
        }

        let mut option_ids = HashSet::with_capacity(schema.options.len());
        for option in &schema.options {
            if !option_ids.insert(option.id.as_str()) {
                return Err(UiComponentDescriptorError::DuplicateSchemaName {
                    component_id: descriptor.id.clone(),
                    schema_kind: "option",
                    name: option.id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn validate_slot_names(
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentDescriptorError> {
    let mut names = HashSet::with_capacity(descriptor.slot_schema.len());
    for slot in &descriptor.slot_schema {
        if slot.name.trim().is_empty() || !names.insert(slot.name.as_str()) {
            return Err(UiComponentDescriptorError::DuplicateSlotName {
                component_id: descriptor.id.clone(),
                name: slot.name.clone(),
            });
        }
    }
    Ok(())
}

fn validate_schema_defaults(
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentDescriptorError> {
    for schema in descriptor
        .prop_schema
        .iter()
        .chain(descriptor.state_schema.iter())
    {
        if let Some(default_value) = &schema.default_value {
            validate_value_kind(
                descriptor,
                &schema.name,
                schema.value_kind,
                default_value.kind(),
            )?;
            validate_finite_value(descriptor, &schema.name, default_value)?;
        }
        if schema.min.is_some_and(|min| !min.is_finite())
            || schema.max.is_some_and(|max| !max.is_finite())
            || matches!((schema.min, schema.max), (Some(min), Some(max)) if min > max)
        {
            return Err(UiComponentDescriptorError::InvalidRange {
                component_id: descriptor.id.clone(),
                name: schema.name.clone(),
            });
        }
        if schema
            .step
            .is_some_and(|step| !step.is_finite() || step <= 0.0)
        {
            return Err(UiComponentDescriptorError::InvalidStep {
                component_id: descriptor.id.clone(),
                name: schema.name.clone(),
            });
        }
    }
    Ok(())
}

fn validate_default_props(
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentDescriptorError> {
    for (name, value) in &descriptor.default_props {
        let Some(schema) = descriptor.prop(name) else {
            return Err(UiComponentDescriptorError::MissingDefaultPropSchema {
                component_id: descriptor.id.clone(),
                name: name.clone(),
            });
        };
        validate_value_kind(descriptor, name, schema.value_kind, value.kind())?;
        validate_finite_value(descriptor, name, value)?;
    }
    Ok(())
}

// 嵌套值沿用所属模式或属性名，诊断定位声明字段而非数组下标或映射子路径。
fn validate_finite_value(
    descriptor: &UiComponentDescriptor,
    name: &str,
    value: &UiValue,
) -> Result<(), UiComponentDescriptorError> {
    match value {
        UiValue::Float(value) => validate_finite_number(descriptor, name, *value),
        UiValue::Vec2(values) => values
            .iter()
            .try_for_each(|value| validate_finite_number(descriptor, name, *value)),
        UiValue::Vec3(values) => values
            .iter()
            .try_for_each(|value| validate_finite_number(descriptor, name, *value)),
        UiValue::Vec4(values) => values
            .iter()
            .try_for_each(|value| validate_finite_number(descriptor, name, *value)),
        UiValue::Array(values) => values
            .iter()
            .try_for_each(|value| validate_finite_value(descriptor, name, value)),
        UiValue::Map(values) => values
            .values()
            .try_for_each(|value| validate_finite_value(descriptor, name, value)),
        _ => Ok(()),
    }
}

fn validate_finite_number(
    descriptor: &UiComponentDescriptor,
    name: &str,
    value: f64,
) -> Result<(), UiComponentDescriptorError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(UiComponentDescriptorError::NonFiniteNumber {
            component_id: descriptor.id.clone(),
            name: name.to_string(),
        })
    }
}

// 模板这里只检查其非空条件和显式槽名；不实例化节点或验证整棵布局。
fn validate_palette(descriptor: &UiComponentDescriptor) -> Result<(), UiComponentDescriptorError> {
    let Some(metadata) = &descriptor.palette else {
        return Ok(());
    };
    if metadata.sort_key.trim().is_empty() {
        return Err(UiComponentDescriptorError::EmptyPaletteSortKey {
            component_id: descriptor.id.clone(),
        });
    }
    if metadata.default_node.is_empty() {
        return Err(UiComponentDescriptorError::MissingPaletteDefaultNode {
            component_id: descriptor.id.clone(),
        });
    }
    if let Some(slot_name) = metadata.default_node.slot_name.as_deref() {
        if descriptor.slot_schema(slot_name).is_none() {
            return Err(UiComponentDescriptorError::UnknownDefaultNodeSlot {
                component_id: descriptor.id.clone(),
                slot_name: slot_name.to_string(),
            });
        }
    }
    Ok(())
}

fn validate_capabilities(
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentDescriptorError> {
    if descriptor
        .required_render_capabilities
        .contains(&UiRenderCapability::VirtualizedLayout)
        && !descriptor
            .required_host_capabilities
            .contains(&UiHostCapability::VirtualizedLayout)
    {
        return Err(
            UiComponentDescriptorError::MissingVirtualizedHostCapability {
                component_id: descriptor.id.clone(),
            },
        );
    }
    Ok(())
}

fn validate_value_kind(
    descriptor: &UiComponentDescriptor,
    name: &str,
    expected: UiValueKind,
    actual: UiValueKind,
) -> Result<(), UiComponentDescriptorError> {
    if expected == UiValueKind::Any || expected == actual {
        return Ok(());
    }
    Err(UiComponentDescriptorError::DefaultValueKindMismatch {
        component_id: descriptor.id.clone(),
        name: name.to_string(),
        expected,
        actual,
    })
}

#[cfg(test)]
#[path = "validation/tests/hash_membership.rs"]
mod hash_membership_tests;

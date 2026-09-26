use std::str::FromStr;

use serde_json::{Map, Value};

use crate::project::{AssetRef, PersistedAssetReference, RelPath};
use crate::resource::{AssetUuid, ResourceLocator, ResourceScheme};
use crate::serialization::MigrateError;

use super::{RetiredAssetRefMigrationBudget, RetiredAssetRefMigrationError, RetiredAssetReference};

/// 仅将精确的旧 `{ uuid, url }` 对象交给调用方解析器；形似对象保持原样。
pub fn migrate_retired_asset_references_with<E>(
    value: Value,
    resolver: impl FnMut(RetiredAssetReference) -> Result<AssetRef, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>> {
    migrate_retired_asset_references_with_budget(
        value,
        RetiredAssetRefMigrationBudget::standard(),
        resolver,
    )
}

/// 在调用方预算内遍历整份 JSON；预算不通过时解析器不会收到任何引用。
pub fn migrate_retired_asset_references_with_budget<E>(
    value: Value,
    budget: RetiredAssetRefMigrationBudget,
    mut resolver: impl FnMut(RetiredAssetReference) -> Result<AssetRef, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>> {
    migrate_value_with_budget(value, budget, &mut resolver)
}

/// 旧版无上下文迁移入口：项目引用按 `res://` 生成路径提示，builtin 引用保留独立种类。
pub fn migrate_retired_asset_references(value: Value) -> Result<Value, MigrateError> {
    migrate_retired_persisted_asset_references_with(value, |reference| {
        let locator = reference.locator();
        if locator.scheme() != ResourceScheme::Res {
            return Err(MigrateError::invalid_payload(format!(
                "retired project asset reference must use res://, found {locator}"
            )));
        }
        let path_hint = RelPath::parse(format!("assets/{}", locator.path()))
            .map_err(|error| MigrateError::invalid_payload(error.to_string()))?;
        AssetRef::try_new(
            reference.guid(),
            path_hint,
            locator.label().map(str::to_string),
        )
        .map_err(|error| MigrateError::invalid_payload(error.to_string()))
    })
    .map_err(flatten_default_error)
}

/// 把旧项目引用交给解析器生成当前资产身份；builtin 引用转为独立的持久化种类。
pub fn migrate_retired_persisted_asset_references_with<E>(
    value: Value,
    project_resolver: impl FnMut(RetiredAssetReference) -> Result<AssetRef, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>> {
    migrate_retired_persisted_asset_references_with_budget(
        value,
        RetiredAssetRefMigrationBudget::standard(),
        project_resolver,
    )
}

/// 以调用方预算迁移项目和 builtin 引用；解析器只处理项目引用。
pub fn migrate_retired_persisted_asset_references_with_budget<E>(
    value: Value,
    budget: RetiredAssetRefMigrationBudget,
    mut project_resolver: impl FnMut(RetiredAssetReference) -> Result<AssetRef, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>> {
    migrate_value_with_budget(value, budget, &mut |reference| {
        if reference.locator().scheme() == ResourceScheme::Builtin {
            return Ok(PersistedAssetReference::builtin(
                reference.locator().clone(),
            ));
        }
        project_resolver(reference).map(PersistedAssetReference::project)
    })
}

/// 只迁移一个精确的旧引用，不遍历其所属容器；格式所有者用它处理与其他字段平铺的引用。
pub fn migrate_retired_persisted_asset_reference_with<E>(
    value: Value,
    mut project_resolver: impl FnMut(RetiredAssetReference) -> Result<AssetRef, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>> {
    let Value::Object(values) = value else {
        return Err(invalid_shape(
            "retired asset reference must be an object".to_string(),
        ));
    };
    if !is_exact_retired_shape(&values) {
        return Err(invalid_shape(
            "retired asset reference must contain exactly uuid and url".to_string(),
        ));
    }
    migrate_reference(values, &mut |reference| {
        if reference.locator().scheme() == ResourceScheme::Builtin {
            return Ok(PersistedAssetReference::builtin(
                reference.locator().clone(),
            ));
        }
        project_resolver(reference).map(PersistedAssetReference::project)
    })
}

fn migrate_value_with_budget<E, R>(
    mut value: Value,
    budget: RetiredAssetRefMigrationBudget,
    resolver: &mut impl FnMut(RetiredAssetReference) -> Result<R, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>>
where
    R: serde::Serialize,
{
    // 先完成整树预算检查，避免解析器已有副作用后才发现超限。
    admit_migration_value(&value, budget)?;

    let mut stack = vec![RewriteFrame::Visit(&mut value)];
    while let Some(frame) = stack.pop() {
        match frame {
            RewriteFrame::Visit(current) => {
                if matches!(current, Value::Object(values) if is_exact_retired_shape(values)) {
                    let Value::Object(values) = std::mem::take(current) else {
                        unreachable!("exact retired references are objects");
                    };
                    *current = migrate_reference(values, resolver)?;
                    continue;
                }
                match current {
                    Value::Array(values) => {
                        stack.push(RewriteFrame::Array(values.iter_mut()));
                    }
                    Value::Object(values) => {
                        stack.push(RewriteFrame::Object(values.iter_mut()));
                    }
                    _ => {}
                }
            }
            RewriteFrame::Array(mut values) => {
                if let Some(value) = values.next() {
                    stack.push(RewriteFrame::Array(values));
                    stack.push(RewriteFrame::Visit(value));
                }
            }
            RewriteFrame::Object(mut values) => {
                if let Some((_, value)) = values.next() {
                    stack.push(RewriteFrame::Object(values));
                    stack.push(RewriteFrame::Visit(value));
                }
            }
        }
    }
    Ok(value)
}

enum RewriteFrame<'value> {
    Visit(&'value mut Value),
    Array(std::slice::IterMut<'value, Value>),
    Object(serde_json::map::IterMut<'value>),
}

fn admit_migration_value<E>(
    value: &Value,
    budget: RetiredAssetRefMigrationBudget,
) -> Result<(), RetiredAssetRefMigrationError<E>> {
    let mut visited_nodes = 0_usize;
    let mut references = 0_usize;
    let mut stack = vec![AdmissionFrame::Visit(value, 0_usize)];
    while let Some(frame) = stack.pop() {
        match frame {
            AdmissionFrame::Visit(current, depth) => {
                visited_nodes = visited_nodes.checked_add(1).unwrap_or(usize::MAX);
                ensure_resource_limit(
                    "retired asset migration nodes",
                    visited_nodes,
                    budget.max_nodes(),
                )?;
                ensure_resource_limit("retired asset migration depth", depth, budget.max_depth())?;
                if matches!(current, Value::Object(values) if is_exact_retired_shape(values)) {
                    references = references.checked_add(1).unwrap_or(usize::MAX);
                    ensure_resource_limit(
                        "retired asset migration references",
                        references,
                        budget.max_references(),
                    )?;
                }
                let child_depth = depth.checked_add(1).unwrap_or(usize::MAX);
                match current {
                    Value::Array(values) => {
                        stack.push(AdmissionFrame::Array(values.iter(), child_depth));
                    }
                    Value::Object(values) => {
                        stack.push(AdmissionFrame::Object(values.iter(), child_depth));
                    }
                    _ => {}
                }
            }
            AdmissionFrame::Array(mut values, depth) => {
                if let Some(value) = values.next() {
                    stack.push(AdmissionFrame::Array(values, depth));
                    stack.push(AdmissionFrame::Visit(value, depth));
                }
            }
            AdmissionFrame::Object(mut values, depth) => {
                if let Some((_, value)) = values.next() {
                    stack.push(AdmissionFrame::Object(values, depth));
                    stack.push(AdmissionFrame::Visit(value, depth));
                }
            }
        }
    }
    Ok(())
}

enum AdmissionFrame<'value> {
    Visit(&'value Value, usize),
    Array(std::slice::Iter<'value, Value>, usize),
    Object(serde_json::map::Iter<'value>, usize),
}

fn ensure_resource_limit<E>(
    resource: &'static str,
    found: usize,
    max: usize,
) -> Result<(), RetiredAssetRefMigrationError<E>> {
    if found > max {
        return Err(RetiredAssetRefMigrationError::ResourceLimitExceeded {
            resource,
            max,
            found,
        });
    }
    Ok(())
}

fn is_exact_retired_shape(values: &Map<String, Value>) -> bool {
    values.len() == 2 && values.contains_key("uuid") && values.contains_key("url")
}

fn migrate_reference<E, R>(
    mut values: Map<String, Value>,
    resolver: &mut impl FnMut(RetiredAssetReference) -> Result<R, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>>
where
    R: serde::Serialize,
{
    let guid = take_string(&mut values, "uuid")?;
    let locator = take_string(&mut values, "url")?;
    let guid = AssetUuid::from_str(&guid).map_err(|error| invalid_shape(error.to_string()))?;
    let locator =
        ResourceLocator::parse(&locator).map_err(|error| invalid_shape(error.to_string()))?;
    let reference = resolver(RetiredAssetReference::new(guid, locator))
        .map_err(RetiredAssetRefMigrationError::Resolve)?;
    serde_json::to_value(reference).map_err(|error| invalid_shape(error.to_string()))
}

fn take_string<E>(
    values: &mut Map<String, Value>,
    field: &'static str,
) -> Result<String, RetiredAssetRefMigrationError<E>> {
    values
        .remove(field)
        .and_then(|value| value.as_str().map(str::to_string))
        .ok_or_else(|| invalid_shape(format!("retired asset {field} must be a string")))
}

fn invalid_shape<E>(message: String) -> RetiredAssetRefMigrationError<E> {
    RetiredAssetRefMigrationError::InvalidShape { message }
}

fn flatten_default_error(error: RetiredAssetRefMigrationError<MigrateError>) -> MigrateError {
    match error {
        RetiredAssetRefMigrationError::InvalidShape { message } => {
            MigrateError::invalid_payload(message)
        }
        RetiredAssetRefMigrationError::ResourceLimitExceeded {
            resource,
            max,
            found,
        } => MigrateError::invalid_payload(format!(
            "retired asset reference migration {resource} limit {max} exceeded (found {found})"
        )),
        RetiredAssetRefMigrationError::Resolve(error) => error,
    }
}

use std::str::FromStr;

use serde_json::{Map, Value};

use crate::project::{AssetRef, PersistedAssetReference, RelPath};
use crate::resource::{AssetUuid, ResourceLocator, ResourceScheme};
use crate::serialization::MigrateError;

use super::{RetiredAssetRefMigrationBudget, RetiredAssetRefMigrationError, RetiredAssetReference};

/// Rewrites only exact retired `{ uuid, url }` objects through the supplied resolver.
/// 仅将精确的旧 `{ uuid, url }` 对象交给调用方解析器；形似对象保持原样。
///
/// # 使用时机与生命周期
///
/// 适用于调用方持有资产身份解析上下文的 JSON 文档迁移。函数先采用标准预算检查输入，
/// 通过后才调用 resolver；之后发生的形状或解析错误不会撤销此前 resolver 的外部副作用。
///
/// # 参数
///
/// `value` 是已载入的 JSON 值；`resolver` 将识别出的旧引用映射为当前 `AssetRef`。
///
/// # 约束
///
/// 非精确形状的对象保留其字段并继续遍历子值；只有精确旧引用会交给 resolver。
/// 更严格的资源上限可通过 [`migrate_retired_asset_references_with_budget`] 提供。
///
/// # 返回值
///
/// 成功时返回重写后的 JSON 值；结构、字段或 resolver 错误会终止并返回对应错误。
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

/// Rewrites exact retired references under caller-owned structural limits.
/// 在调用方预算内遍历整份 JSON；预算不通过时解析器不会收到任何引用。
///
/// # 参数
///
/// `budget` 分别限制 JSON 值节点数、根为 0 的嵌套深度和精确旧引用数。
/// `resolver` 为通过精确形状识别并完成 GUID/locator 解析的引用生成 `AssetRef`。
///
/// # 使用时机与生命周期
///
/// 预算阶段先于重写阶段完成；若返回资源超限错误，本次调用尚未执行 resolver。
/// 形状解析或 resolver 错误在重写期间产生，之前已执行的 resolver 副作用不会回滚。
///
/// # 返回值
///
/// 成功时返回改写后的 JSON 值；任一精确引用无法解析或 resolver 失败时返回首个错误。
pub fn migrate_retired_asset_references_with_budget<E>(
    value: Value,
    budget: RetiredAssetRefMigrationBudget,
    mut resolver: impl FnMut(RetiredAssetReference) -> Result<AssetRef, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>> {
    migrate_value_with_budget(value, budget, &mut resolver)
}

/// Plan 11's context-free v0-to-v1 rule for `res://` project references.
/// 旧版无上下文迁移入口：项目引用按 `res://` 生成路径提示，builtin 引用保留独立种类。
///
/// # 参数
///
/// `value` 是已载入的旧版 JSON 值；本入口不读取文件。
///
/// # 返回值
///
/// 成功时返回当前项目/Builtin 持久化引用表示；解析或校验失败时返回 `MigrateError`。
///
/// # 约束
///
/// 项目引用要求定位符使用 `res://`，生成的 `assets/<path>` 仅作为路径提示；
/// 本函数不查询文件系统或注册表。builtin 定位符直接保留为独立持久化变体。
///
/// # 错误
///
/// 输入中的项目定位符使用其他 scheme、路径提示无法通过 RelPath 校验，或引用字段形状无效时返回迁移错误。
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

/// Rewrites project references to `Project(AssetRef)` and retired builtin references to
/// the distinct `Builtin { locator }` authoring contract.
/// 把旧项目引用交给解析器生成当前资产身份；builtin 引用转为独立的持久化种类。
///
/// # 参数
///
/// `value` 是已载入的 JSON 值。`project_resolver` 只接收非 builtin 的旧引用；builtin 变体由本函数直接构造。
/// 其他 scheme 是否可接受由解析器决定。
///
/// # 约束
///
/// 使用标准资源预算；预算通过后解析器才会运行。后续形状或解析错误不会回滚此前回调副作用。
///
/// # 返回值
///
/// 返回包含 project 或 builtin 持久化引用的 JSON 值。
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

/// Rewrites project and builtin retired references under caller-owned structural limits.
/// 以调用方预算迁移项目和 builtin 引用；解析器只处理项目引用。
///
/// # 参数
///
/// `budget` 限制整棵 JSON 值树、嵌套深度和精确旧引用数；builtin 引用不传给 resolver。
/// `project_resolver` 为精确的非 builtin 旧引用生成当前 `AssetRef`。
///
/// # 约束
///
/// 整树预算准入先于任何 resolver 调用；资源超限不会触发回调。后续解析错误可能发生在其他引用已解析之后。
///
/// # 返回值
///
/// 成功时返回重写后的 JSON 值；预算、旧字段解析或 resolver 错误会中止当前结果。
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

/// Rewrites one exact retired `{ uuid, url }` object without traversing its container.
///
/// Format owners use this primitive when a reference is flattened beside owner-specific
/// fields that must not be interpreted by the generic recursive walker.
/// 只迁移一个精确的旧引用，不遍历其所属容器；格式所有者用它处理与其他字段平铺的引用。
///
/// # 参数
///
/// `value` 必须是根对象且只含 `uuid`、`url` 两个字段；项目定位符交由
/// `project_resolver` 解析，builtin 定位符直接成为 builtin 变体。
///
/// # 约束
///
/// 此入口不遍历子值，也不执行批量 JSON 预算检查。解析器副作用不会因后续输出转换错误而回滚。
///
/// # 返回值
///
/// 返回序列化后的当前项目或 builtin 引用；根值形状、字段语法或 resolver 失败时返回错误。
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
    // 预算预检只统计结构，不解析旧引用；成功后才开始可调用 resolver 的改写阶段。
    // 因而资源超限保证无 resolver 副作用，但后续形状或解析错误不会回滚此前回调。
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

// 这些帧通过显式栈逐个访问容器子值，以深度优先遍历而不消耗 Rust 递归栈。
enum RewriteFrame<'value> {
    Visit(&'value mut Value),
    Array(std::slice::IterMut<'value, Value>),
    Object(serde_json::map::IterMut<'value>),
}

fn admit_migration_value<E>(
    value: &Value,
    budget: RetiredAssetRefMigrationBudget,
) -> Result<(), RetiredAssetRefMigrationError<E>> {
    // 预算成功时检查整棵 Value 树；超限可提前退出。此阶段不解析字段，也不调用 resolver。
    let mut visited_nodes = 0_usize;
    let mut references = 0_usize;
    let mut stack = vec![AdmissionFrame::Visit(value, 0_usize)];
    while let Some(frame) = stack.pop() {
        match frame {
            AdmissionFrame::Visit(current, depth) => {
                visited_nodes = visited_nodes.saturating_add(1);
                ensure_resource_limit(
                    "retired asset migration nodes",
                    visited_nodes,
                    budget.max_nodes(),
                )?;
                ensure_resource_limit("retired asset migration depth", depth, budget.max_depth())?;
                if matches!(current, Value::Object(values) if is_exact_retired_shape(values)) {
                    references = references.saturating_add(1);
                    ensure_resource_limit(
                        "retired asset migration references",
                        references,
                        budget.max_references(),
                    )?;
                }
                let child_depth = depth.saturating_add(1);
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

// AdmissionFrame 用显式迭代器保存遍历状态；子值深度入栈时加一，根值从 0 开始。
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
    // 上限为包含式：仅 found 大于 max 时拒绝。
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
    // 这里只识别键集合；字符串类型、UUID 与 locator 语法留给 migrate_reference 校验。
    values.len() == 2 && values.contains_key("uuid") && values.contains_key("url")
}

fn migrate_reference<E, R>(
    mut values: Map<String, Value>,
    resolver: &mut impl FnMut(RetiredAssetReference) -> Result<R, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>>
where
    R: serde::Serialize,
{
    // 先解析两个字符串，再调用 resolver；输出 JSON 序列化若失败，不会撤销该回调的副作用。
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

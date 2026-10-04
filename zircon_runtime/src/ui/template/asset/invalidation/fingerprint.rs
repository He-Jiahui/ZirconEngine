//! 将编译缓存的文档、导入、组件契约和资源引用输入投影为可比较修订。
//! 指纹表示输入身份，不读取资源内容，也不代替资源 watch 的更新事件。

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::Serialize;

use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetFingerprint, UiResourceRef,
};

use super::super::resource_ref::{
    collect_document_resource_dependencies, unique_resource_references,
};

/// 保留导入声明名称到文档内容修订的对应；调用者先完成导入解析与注册。
pub fn document_import_fingerprints(
    imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<BTreeMap<String, UiAssetFingerprint>, UiAssetError> {
    imports
        .iter()
        .map(|(reference, document)| Ok((reference.clone(), fingerprint_document(document)?)))
        .collect()
}

/// 单独记录声明列表，避免尚未注册或移除的导入被文档集合指纹掩盖。
pub fn declared_imports_fingerprint(
    imports: &[String],
) -> Result<UiAssetFingerprint, UiAssetError> {
    fingerprint_serializable(&UiDeclaredImportsFingerprintInput { imports })
}

/// 对拥有确定性集合顺序的源文档求内容修订，供编译缓存匹配根输入。
pub fn fingerprint_document(
    document: &UiAssetDocument,
) -> Result<UiAssetFingerprint, UiAssetError> {
    fingerprint_serializable(document)
}

/// 只隔离根与 widget 导入的公开组件契约变化；组件实现变化由各文档指纹覆盖。
pub fn component_contract_fingerprint(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<UiAssetFingerprint, UiAssetError> {
    let mut source = String::new();
    let mut serializer_buffer = toml::ser::Buffer::new();
    append_contracts(&mut source, &mut serializer_buffer, "root", document)?;
    for (reference, import) in widget_imports {
        append_contracts(&mut source, &mut serializer_buffer, reference, import)?;
    }
    Ok(UiAssetFingerprint::from_bytes(source.as_bytes()))
}

/// 按完整资源引用集合生成修订；同一引用的使用位置和次数不改变这个独立修订。
pub fn resource_dependencies_fingerprint(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    style_imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<UiAssetFingerprint, UiAssetError> {
    let report = collect_document_resource_dependencies(document, widget_imports, style_imports)?;
    let input = UiResourceDependencyFingerprintInput {
        references: unique_resource_references(&report.dependencies)
            .into_iter()
            .collect(),
    };
    fingerprint_serializable(&input)
}

#[derive(Serialize)]
struct UiResourceDependencyFingerprintInput {
    references: Vec<UiResourceRef>,
}

#[derive(Serialize)]
struct UiDeclaredImportsFingerprintInput<'a> {
    imports: &'a [String],
}

// 名称和拥有者分隔不同文档的组件契约，避免同名组件跨导入复用同一身份。
fn append_contracts(
    source: &mut String,
    serializer_buffer: &mut toml::ser::Buffer,
    owner: &str,
    document: &UiAssetDocument,
) -> Result<(), UiAssetError> {
    source.push_str(owner);
    source.push('\n');
    for (component_name, component) in &document.components {
        source.push_str(component_name);
        source.push('\n');
        append_serializable_for_fingerprint(source, serializer_buffer, &component.contract)?;
        source.push('\n');
    }
    Ok(())
}

fn append_serializable_for_fingerprint<T>(
    source: &mut String,
    serializer_buffer: &mut toml::ser::Buffer,
    value: &T,
) -> Result<(), UiAssetError>
where
    T: Serialize,
{
    serializer_buffer.clear();
    value
        .serialize(toml::Serializer::new(serializer_buffer))
        .map_err(fingerprint_serialization_error)?;
    write!(source, "{serializer_buffer}").expect("writing to a String is infallible");
    Ok(())
}

fn fingerprint_serializable<T>(value: &T) -> Result<UiAssetFingerprint, UiAssetError>
where
    T: Serialize,
{
    serialize_for_fingerprint(value)
        .map(|serialized| UiAssetFingerprint::from_bytes(serialized.as_bytes()))
}

fn serialize_for_fingerprint<T>(value: &T) -> Result<String, UiAssetError>
where
    T: Serialize,
{
    toml::to_string(value).map_err(fingerprint_serialization_error)
}

fn fingerprint_serialization_error(error: toml::ser::Error) -> UiAssetError {
    UiAssetError::InvalidDocument {
        asset_id: "ui-asset-fingerprint".to_string(),
        detail: format!("failed to serialize deterministic fingerprint input: {error}"),
    }
}

// 等价测试固定实际序列化输出；被忽略的 release 对照只度量中间输出字符串的消除。
#[cfg(test)]
#[path = "tests/fingerprint_performance_tests.rs"]
mod performance_tests;

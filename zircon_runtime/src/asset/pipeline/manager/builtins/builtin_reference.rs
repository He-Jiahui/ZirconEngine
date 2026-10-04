use crate::asset::{AssetReference, AssetUri};

// 调用方只传入内建资源中的固定定位符；解析失败表示内建资源声明本身无效。
pub(super) fn builtin_reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).expect("builtin asset reference"))
}

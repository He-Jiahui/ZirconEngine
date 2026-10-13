//! `ZrReflect` 派生宏的公开入口；属性解析与字段 token 生成由私有子模块负责。
//! 派生结果实现 `zircon_runtime_interface::reflect::ZrReflect`，供运行时注册与字段适配器调用。

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[path = "attributes.rs"]
mod attributes;
#[path = "derive.rs"]
mod derive;
#[path = "fields.rs"]
mod fields;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

/// 为结构体或枚举生成反射元数据，以及按字段名和当期槽位访问的实现。
/// 此实现仅支持无泛型类型，并同时生成按字段名和当期槽位读写字段的方法。
///
/// # 使用时机与生命周期
/// Rust 编译派生属性时调用此入口；返回的实现与目标类型一同编译，注册元数据在运行时被请求时构造。
///
/// # 参数
/// `input` 是目标项的 token 输入，须为可解析的 Rust 结构体或枚举，并可带 `zr_reflect` 容器及字段属性。
///
/// # 返回值
/// 返回实现 `zircon_runtime_interface::reflect::ZrReflect` 的 token；属性或类型不受支持时返回编译错误 token。
///
/// # 约束
/// 泛型项和 union 不受支持。自动读取或默认自动读写的字段须实现 `ZrReflectValue`；默认成员写入还要求字段类型实现 `PartialEq`，
/// 以便报告值是否变化。自定义 `read`/`write` 路径须满足运行时 trait 的返回类型。
///
/// # 参考
/// 展开逻辑位于 `crate::derive::derive_zr_reflect_impl`；运行时契约为 `zircon_runtime_interface::reflect::ZrReflect`。
#[proc_macro_derive(ZrReflect, attributes(zr_reflect))]
pub fn derive_zr_reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    derive::derive_zr_reflect_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

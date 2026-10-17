//! 为 Runtime 的脚本宿主接口生成类型、函数和内联模块的描述代码。
//!
//! 宏在编译期解析 Rust 定义；生成的类型描述与函数工厂由调用方在运行期使用，模块注册由显式的注册函数调用触发。
//!
//! 生成代码通过 `::zircon_runtime` 访问 framework 合同，消费 crate 需使 Runtime 在该名称下可达。

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[path = "args.rs"]
mod args;
#[path = "attrs.rs"]
mod attrs;
#[path = "derive_type.rs"]
mod derive_type;
#[path = "function.rs"]
mod function;
#[path = "module.rs"]
mod module;
#[path = "tokens.rs"]
mod tokens;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

/// 为结构体或枚举生成 `ZirconScriptType` 的反射注册描述和脚本类型投影。
///
/// ## 参数
///
/// `input` 是派生目标的 Rust 定义。类型级 `zircon_script` 属性支持 `name`、`identity`、`value_kind`、`prototype`、`allow_value_construction` 和 `documentation`（或 `doc`）；字段级支持 `name`、`identity`、`alias`、`type_name`、`value_kind`、`documentation`（或 `doc`）和 `skip`。字符串、路径和布尔参数按各自类型解析。
///
/// ## 返回
///
/// 返回实现 `reflect_type_registration` 与 `script_host_type_projection` 的编译器 token；输入解析或生成失败时返回 `compile_error` token。
///
/// ## 约束与生命周期
///
/// 非空泛型参数和 union 在展开阶段被拒绝；枚举生成 Enum 形状和空字段集合。生成方法每次调用构造拥有数据的描述，类型路径及投影的一致性错误由调用这些方法的代码接收。字段身份键用于稳定 ID；需要跨显示名称变化保持 ID 时，应保留显式的类型与字段 `identity`。值构造选项写入投影描述，实例适配器需另行提供。
///
/// ## 参考
///
/// 参阅 Runtime framework 中 `ZirconScriptType::script_host_type_descriptor`、`ScriptHostTypeProjection`，以及 Interface 的 `ReflectTypeRegistration`、`ReflectFieldId::from_stable_keys`。
#[proc_macro_derive(ZirconScriptType, attributes(zircon_script))]
pub fn derive_zircon_script_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    derive_type::derive_zircon_script_type_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// 保留宿主 Rust 函数，并生成同作用域的函数描述工厂与同步调用包装工厂。
///
/// ## 参数
///
/// `attr` 支持 `name`、`return_type_name`、`return_value_kind`、可重复的 `capability` 与 `documentation`（或 `doc`）；返回类型种类使用 Rust 路径，其余使用字符串。`item` 是函数定义；参数须为 `Pat::Ident` 模式，receiver、async 和非空泛型参数被拒绝。
///
/// ## 返回
///
/// 返回原函数与两个私有 helper 的 token；解析或生成错误转为编译诊断。描述工厂声明固定参数数量及类型，导出工厂创建 `HostExportFunction`。
/// 消费 crate 须启用 Runtime 的 `script` feature，以提供生成的 VM 导出类型。
///
/// ## 约束与生命周期
///
/// 执行包装按参数索引同步调用 `ScriptHostFromArgument`，任一转换失败便返回错误；转换所得 Rust 值不依赖当前参数视图的短期借用。显式返回通过 `ScriptHostIntoValue` 生成拥有的 `ScriptHostValue`，省略返回类型时包装返回 Null。返回元数据覆盖修改描述，实际值仍走转换 trait。注册表持有回调的 Arc；参数数量和 capability 检查在调用表进入回调前进行，原函数仅在转换全部成功后调用。
///
/// ## 参考
///
/// 参阅 Runtime 的 `HostExportRegistry::register_module`、`ScriptCallSite::call`、`ScriptHostArguments::with_argument` 与两个转换 trait。
#[proc_macro_attribute]
pub fn zircon_host_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as args::HostFunctionArgs);
    let item = parse_macro_input!(item as syn::ItemFn);
    function::host_function_impl(args, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// 为内联模块生成汇总类型和函数的描述函数，以及显式注册函数。
///
/// ## 参数
///
/// `attr` 必须包含字符串 `name`，可选字符串 `version`、可重复的 `capability` 和 `documentation`（或 `doc`）；版本默认 `0.1.0`。`item` 是内联模块定义，收集范围为模块直接子项中的宿主函数和派生脚本类型。
///
/// ## 返回
///
/// 返回保留原模块内容的 token，并在模块内增加 `<模块名>_host_module_descriptor` 和 `register_<模块名>_host_module`；解析或生成错误转为编译诊断。
/// 消费 crate 须启用 Runtime 的 `script` feature，以提供生成的 VM 错误和注册表类型。
///
/// ## 约束与生命周期
///
/// 调用描述函数时才求取类型和函数元数据，类型描述错误转换为 `VmError::Operation` 并向上传播。调用注册函数时先构造描述，再把描述和回调交给传入的 `HostExportRegistry`；描述构造失败时注册不会发生。名称、能力声明和回调匹配由注册接纳检查验证；接纳成功返回 `HostHandle`，拥有描述和回调由注册表保存。
///
/// ## 参考
///
/// 参阅 Runtime framework 的 `ZirconScriptType::script_host_type_descriptor` 和 Runtime VM 的 `HostExportRegistry::register_module`。
#[proc_macro_attribute]
pub fn zircon_host_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as args::HostModuleArgs);
    let item = parse_macro_input!(item as syn::ItemMod);
    module::host_module_impl(args, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

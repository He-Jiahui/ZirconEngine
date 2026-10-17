//! 归一化实体与虚拟字段，并生成字段元数据及读写分派所需的 token。
//! 派生主流程使用同一字段序列构造注册字段列表和当前槽位访问器。

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Field, GenericArgument, Index, LitInt, Member, PathArguments, Type};

use crate::attributes::{parse_field_attributes, FieldAttributes};

/// 一个已经过属性解析并归一化、供单次 derive 展开生成元数据与访问分支的字段。
///
/// `crate::attributes` 先产出 [`FieldAttributes`]，实体字段和虚拟字段分别经 [`collect_fields`]、
/// [`collect_virtual_fields`] 转成此记录；随后 [`crate::derive::derive_zr_reflect_impl`] 将同一序列交给
/// 元数据、名称分派和槽位分派生成器。该记录只在编译期 token 构造期间存在。
pub(crate) struct ReflectedField {
    /// 对工具和按名访问器公开的规范字段名。
    pub(crate) name: String,
    /// 参与稳定字段 ID 计算的字段键；缺省由配置构建阶段设为 `name`。
    pub(crate) identity: String,
    /// 随字段元数据导出的兼容别名；不改变生成分支的规范字段名。
    pub(crate) aliases: Vec<String>,
    /// 实体字段的结构体成员；虚拟字段没有成员。
    pub(crate) member: Option<Member>,
    /// 实体字段的 Rust 类型；虚拟字段通过访问器与反射值交互。
    pub(crate) ty: Option<Type>,
    /// 反射 schema 中声明的值类型路径。
    pub(crate) value_type_path: String,
    /// 已解析的 runtime-interface 编辑器提示变体。
    pub(crate) editor_hint: syn::Ident,
    /// 是否拒绝生成的反射写入。
    pub(crate) readonly: bool,
    /// 导出到字段注册元数据的序列化标记。
    pub(crate) serializable: bool,
    /// 导出到字段注册元数据的编辑器可见标记。
    pub(crate) editor_visible: bool,
    /// 可选的自定义读访问器；提供后优先于实体成员转换。
    pub(crate) read: Option<syn::Path>,
    /// 可选的自定义写访问器；提供后优先于实体成员转换。
    pub(crate) write: Option<syn::Path>,
}

/// 将容器属性声明的虚拟字段转换为访问器驱动的规范字段。
///
/// # 使用时机与生命周期
/// 派生主流程在实体字段之后调用；结果仅用于本次宏展开的元数据及分派 token。
///
/// # 参数
/// `fields` 是容器属性解析出的虚拟字段配置，调用前已检查必填键和只读写入冲突。
///
/// # 返回值
/// 返回虚拟字段序列；每个字段没有 Rust 成员和 Rust 字段类型，必须由属性提供读回调及反射值类型路径。
///
/// # 约束
/// 可编辑虚拟字段还须提供写回调；编辑器提示必须属于 `parse_editor_hint` 接受的集合。
///
/// # 参考
/// 调用点为 [`crate::derive::derive_zr_reflect_impl`]；回调展开由 [`read_arm_tokens`] 和 [`write_arm_tokens`] 处理。
pub(crate) fn collect_virtual_fields(
    fields: Vec<FieldAttributes>,
) -> syn::Result<Vec<ReflectedField>> {
    let mut reflected = Vec::with_capacity(fields.len());
    for field in fields {
        let name = field.name.clone().ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "virtual reflected field is missing its name",
            )
        })?;
        let value_type_path = field.value_type_path.clone().ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "virtual reflected field is missing its value type path",
            )
        })?;
        reflected.push(build_reflected_field(
            name,
            None,
            None,
            value_type_path,
            field,
            proc_macro2::Span::call_site(),
        )?);
    }
    Ok(reflected)
}

// 跳过字段后压紧导出顺序，使生成的槽位与注册元数据的字段序列一一对应。
/// 收集结构体或元组字段，过滤 `skip` 字段并按源声明顺序保留其余字段。
///
/// # 参数
/// `fields` 是目标结构体的 `syn::Fields`，涵盖具名、元组和单元字段形状。
///
/// # 返回值
/// 返回带 Rust 成员及类型信息的规范字段序列；属性、名称或未支持值类型无效时返回 `syn::Error`。
///
/// # 参考
/// 由 [`crate::derive::derive_zr_reflect_impl`] 用于结构体；对应属性解析由 [`crate::attributes::parse_field_attributes`] 完成。
pub(crate) fn collect_fields(fields: &syn::Fields) -> syn::Result<Vec<ReflectedField>> {
    let mut reflected = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let attributes = parse_field_attributes(&field.attrs)?;
        if attributes.skip {
            continue;
        }
        reflected.push(reflected_field(field, index, attributes)?);
    }
    Ok(reflected)
}

/// 生成单个字段的注册元数据表达式，包括稳定 ID、类型、别名和可见/可写标记。
///
/// # 使用时机与生命周期
/// 派生主流程为规范字段序列中的每个字段调用；返回表达式嵌入 trait 方法，运行时调用注册方法时才求值。
///
/// # 参数
/// `field` 是已归一化字段；`owner_identity` 是表示类型身份键的 token 表达式。
/// 类型身份键与 `field.identity` 共同作为稳定 ID 输入，名称和槽位描述当前 schema。
///
/// # 返回值
/// 返回构造 `ReflectFieldInfo` 的 token，并附加别名、可编辑、可序列化和编辑器可见标记。
///
/// # 约束
/// 输入身份键须已由属性和类型路径处理逻辑选定；本函数不重新验证稳定键，也不分配槽位。
///
/// # 参考
/// 输出使用 `ReflectFieldId::from_stable_keys`；调用者与读写分派共享 `ReflectedField` 序列。
pub(crate) fn field_info_tokens(
    field: &ReflectedField,
    owner_identity: &TokenStream,
) -> TokenStream {
    let name = &field.name;
    let identity = &field.identity;
    let aliases = field.aliases.iter().map(|alias| quote!(#alias.to_string()));
    let value_type_path = &field.value_type_path;
    let editor_hint = &field.editor_hint;
    let editable = !field.readonly;
    let serializable = field.serializable;
    let editor_visible = field.editor_visible;
    quote! {
        ::zircon_runtime_interface::reflect::ReflectFieldInfo::new(
            ::zircon_runtime_interface::reflect::ReflectFieldId::from_stable_keys(
                #owner_identity,
                #identity,
            ),
            #name,
            #value_type_path,
            ::zircon_runtime_interface::reflect::ReflectEditorHint::#editor_hint,
        )
        .with_aliases(vec![#(#aliases),*])
        .with_editable(#editable)
        .with_serializable(#serializable)
        .with_editor_visible(#editor_visible)
    }
}

/// 为按规范字段名读取生成一个匹配分支；自定义回调优先，否则调用字段值转换 trait。
///
/// # 使用时机与生命周期
/// derive 构造 `ZrReflect::read_reflected_field` 的 `match` 时调用；生成分支在每次运行时字段读取中执行。
///
/// # 参数
/// `field` 是已归一化字段，提供匹配名以及成员/类型或自定义读路径。
///
/// # 返回值
/// 返回匹配臂 token；臂产生 `Result<ReflectedValue, ReflectError>`，缺失必要虚拟字段读访问器时生成编译错误。
///
/// # 约束
/// 自定义路径须可用 `&self` 调用并返回 trait 约定的结果；默认成员路径要求 `member` 与 `ty` 同时存在，且 `ty` 实现 `ZrReflectValue`。
///
/// # 参考
/// 按槽位的对应生成器为 [`read_slot_arm_tokens`]；trait 方法由 `crate::derive::derive_zr_reflect_impl` 展开。
pub(crate) fn read_arm_tokens(field: &ReflectedField) -> TokenStream {
    let name = &field.name;
    if let Some(read) = &field.read {
        return quote!(#name => #read(self),);
    }
    let (Some(member), Some(ty)) = (&field.member, &field.ty) else {
        return quote!(compile_error!("virtual reflected fields require a read accessor"););
    };
    quote! {
        #name => Ok(
            <#ty as ::zircon_runtime_interface::reflect::ZrReflectValue>::to_reflected_value(
                &self.#member,
            ),
        ),
    }
}

/// 为当前字段槽位生成读取分支；槽位只在当前注册字段顺序内有效。
///
/// # 使用时机与生命周期
/// derive 生成 `ZrReflect::read_reflected_field_by_slot` 的 `match` 时调用；槽位索引来自同一规范字段序列。
/// 注册顺序变化时槽位可能变化，长期保存的引用应使用稳定字段 ID。
///
/// # 参数
/// `field` 是已归一化字段；`index` 是该字段在未跳过实体字段与虚拟字段合并序列中的零基索引。
///
/// # 返回值
/// 返回 `u32` 槽位匹配臂 token，读取成功或失败的类型与 [`read_arm_tokens`] 相同。
///
/// # 约束
/// 自定义读路径和默认成员路径的要求与 [`read_arm_tokens`] 相同。
///
/// # 参考
/// 槽位契约见 `zircon_runtime_interface::reflect::ZrReflect::read_reflected_field_by_slot`。
pub(crate) fn read_slot_arm_tokens(field: &ReflectedField, index: usize) -> TokenStream {
    let slot = slot_literal(index);
    if let Some(read) = &field.read {
        return quote!(#slot => #read(self),);
    }
    let (Some(member), Some(ty)) = (&field.member, &field.ty) else {
        return quote!(compile_error!("virtual reflected fields require a read accessor"););
    };
    quote! {
        #slot => Ok(
            <#ty as ::zircon_runtime_interface::reflect::ZrReflectValue>::to_reflected_value(
                &self.#member,
            ),
        ),
    }
}

/// 为按规范字段名写入生成分支，保留只读错误、用户回调和默认类型转换路径。
///
/// # 使用时机与生命周期
/// derive 构造 `ZrReflect::write_reflected_field` 的 `match` 时调用；生成臂在每次运行时写入请求中执行。
///
/// # 参数
/// `field` 是已归一化字段；`type_path` 是用于类型转换和错误上下文的 token 表达式。
///
/// # 返回值
/// 返回匹配臂 token，结果为 `Result<bool, ReflectError>`；只读臂返回 `NonEditableField`。
///
/// # 约束
/// 自定义写路径须接受 `(&mut Self, ReflectedValue)` 并返回 trait 约定的结果。默认成员写入要求成员类型实现
/// `ZrReflectValue + PartialEq`，以便先完成反射值转换，再判断是否需要赋值并返回变化标记。
/// 虚拟可写字段须有自定义写路径。
///
/// # 参考
/// 按槽位生成器为 [`write_slot_arm_tokens`]；运行时适配器仅在返回 `true` 时提交克隆后的组件。
pub(crate) fn write_arm_tokens(field: &ReflectedField, type_path: &TokenStream) -> TokenStream {
    let name = &field.name;
    if field.readonly {
        return quote! {
            #name => Err(::zircon_runtime_interface::reflect::ReflectError::NonEditableField {
                type_path: (#type_path).to_string(),
                field_name: field_name.to_string(),
            })
        };
    }
    if let Some(write) = &field.write {
        return quote!(#name => #write(self, value));
    }

    let (Some(member), Some(ty)) = (&field.member, &field.ty) else {
        return quote!(compile_error!("editable virtual reflected fields require a write accessor"););
    };
    quote! {
        #name => {
            let next = <#ty as ::zircon_runtime_interface::reflect::ZrReflectValue>::from_reflected_value(
                value,
                #type_path,
                field_name,
            )?;
            if self.#member == next {
                Ok(false)
            } else {
                self.#member = next;
                Ok(true)
            }
        }
    }
}

/// 为当前字段槽位生成写入分支；槽位只在当前注册字段顺序内有效。
///
/// # 使用时机与生命周期
/// derive 构造 `ZrReflect::write_reflected_field_by_slot` 的 `match` 时调用；槽位在 schema 顺序变化后可能变化。
///
/// # 参数
/// `field` 是已归一化字段，`index` 是规范序列中的零基索引，`type_path` 是错误上下文及值转换 token。
///
/// # 返回值
/// 返回 `u32` 槽位匹配臂 token，结果为 `Result<bool, ReflectError>`；错误以字段规范名表示目标。
///
/// # 约束
/// 只读字段拒绝写入；自定义写路径和默认成员路径的类型约束与 [`write_arm_tokens`] 相同。
///
/// # 参考
/// 稳定字段 ID 到当期槽位的解析由 schema catalog 执行；trait 契约为 `zircon_runtime_interface::reflect::ZrReflect::write_reflected_field_by_slot`。
pub(crate) fn write_slot_arm_tokens(
    field: &ReflectedField,
    index: usize,
    type_path: &TokenStream,
) -> TokenStream {
    let slot = slot_literal(index);
    let name = &field.name;
    if field.readonly {
        return quote! {
            #slot => Err(::zircon_runtime_interface::reflect::ReflectError::NonEditableField {
                type_path: (#type_path).to_string(),
                field_name: #name.to_string(),
            })
        };
    }
    if let Some(write) = &field.write {
        return quote!(#slot => #write(self, value));
    }

    let (Some(member), Some(ty)) = (&field.member, &field.ty) else {
        return quote!(compile_error!("editable virtual reflected fields require a write accessor"););
    };
    quote! {
        #slot => {
            let next = <#ty as ::zircon_runtime_interface::reflect::ZrReflectValue>::from_reflected_value(
                value,
                #type_path,
                #name,
            )?;
            if self.#member == next {
                Ok(false)
            } else {
                self.#member = next;
                Ok(true)
            }
        }
    }
}

/// 将 Rust 序列索引表示为 trait 约定的 `u32` 槽位字面量。
///
/// # 参数
/// `index` 是过滤 `skip` 字段并追加虚拟字段后的零基索引。
///
/// # 返回值
/// 返回带 `u32` 后缀的整数 token，供按槽位读写匹配臂共用。
///
/// # 约束
/// 调用者须使用注册元数据的同一序列生成索引；槽位只对当前 schema 顺序有意义。
///
/// # 参考
/// 调用点为 [`read_slot_arm_tokens`] 和 [`write_slot_arm_tokens`]。
fn slot_literal(index: usize) -> LitInt {
    LitInt::new(&format!("{index}u32"), proc_macro2::Span::call_site())
}

/// 从一个未跳过的 Rust 字段构造反射字段，优先使用显式类型路径，必要时才推断。
///
/// `index` 是原始结构体字段索引，因此 tuple 字段在跳过前仍保留正确成员编号；输出槽位随后按收集结果压紧。
///
/// # 使用时机与生命周期
/// `collect_fields` 在排除 `skip` 字段后逐项调用；返回记录只参与本次 derive 的元数据和访问分支生成。
///
/// # 参数
/// `field` 是原始 Rust 字段，`attributes` 是已解析的字段选项，`index` 是过滤前的声明序号。
///
/// # 返回值
/// 返回带成员、Rust 类型和值类型路径的规范字段；空名称、空显式类型路径、无法推断的类型或无效提示会返回带字段位置的 `syn::Error`。
fn reflected_field(
    field: &Field,
    index: usize,
    attributes: FieldAttributes,
) -> syn::Result<ReflectedField> {
    let member = field
        .ident
        .clone()
        .map(Member::Named)
        .unwrap_or_else(|| Member::Unnamed(Index::from(index)));
    let name = attributes.name.clone().unwrap_or_else(|| match &member {
        Member::Named(ident) => ident.to_string(),
        Member::Unnamed(index) => index.index.to_string(),
    });
    if name.trim().is_empty() {
        return Err(syn::Error::new(
            field.span(),
            "reflected field name must not be empty",
        ));
    }
    let value_type_path = match attributes.value_type_path.clone() {
        Some(path) if !path.trim().is_empty() => path,
        Some(_) => {
            return Err(syn::Error::new(
                field.span(),
                "reflected field value_type_path must not be empty",
            ))
        }
        None => infer_value_type_path(&field.ty).ok_or_else(|| {
            syn::Error::new(
                field.ty.span(),
                "reflected field type requires value_type_path = \"...\"",
            )
        })?,
    };
    build_reflected_field(
        name,
        Some(member),
        Some(field.ty.clone()),
        value_type_path,
        attributes,
        field.span(),
    )
}

/// 合并字段名、身份键、类型路径和编辑器提示，生成元数据及访问器共享的字段记录。
///
/// `member` 与 `ty` 同时为 `None` 表示虚拟字段；其读写由可选访问器路径处理。
///
/// # 使用时机与生命周期
/// `collect_virtual_fields` 和 `reflected_field` 共用此构造步骤；结果只保存在当前展开的规范字段序列中。
///
/// # 参数
/// `name`、`value_type_path` 和 `attributes` 提供解析后的字段配置，`member` 与 `ty` 描述实体成员，`span` 用于提示名错误定位。
///
/// # 返回值
/// 返回供元数据及访问分支共用的字段记录；缺省身份键取字段名，提示缺省时按值类型推导，可序列化和编辑器可见默认均为 `true`；不支持的提示返回 `syn::Error`。
fn build_reflected_field(
    name: String,
    member: Option<Member>,
    ty: Option<Type>,
    value_type_path: String,
    attributes: FieldAttributes,
    span: proc_macro2::Span,
) -> syn::Result<ReflectedField> {
    let identity = attributes.identity.clone().unwrap_or_else(|| name.clone());
    let editor_hint_name = attributes
        .editor_hint
        .clone()
        .unwrap_or_else(|| inferred_editor_hint(&value_type_path).to_string());
    let editor_hint = parse_editor_hint(&editor_hint_name, span)?;
    Ok(ReflectedField {
        name,
        identity,
        aliases: attributes.aliases,
        member,
        ty,
        value_type_path,
        editor_hint,
        readonly: attributes.readonly,
        serializable: attributes.serializable.unwrap_or(true),
        editor_visible: attributes.editor_visible.unwrap_or(true),
        read: attributes.read,
        write: attributes.write,
    })
}

/// 对受支持的 Rust 路径末段推断统一反射值类型；未知类型或非路径类型返回 `None`。
///
/// 列表只在其唯一类型实参也可推断时递归得到 `List<...>`；别名和自定义类型应显式给出路径。
fn infer_value_type_path(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) => {
            let segment = path.path.segments.last()?;
            match segment.ident.to_string().as_str() {
                "bool" => Some("Bool".to_string()),
                "i8" | "i16" | "i32" | "i64" => Some("Integer".to_string()),
                "u8" | "u16" | "u32" | "u64" => Some("Unsigned".to_string()),
                "f32" | "Real" => Some("Scalar".to_string()),
                "String" | "str" => Some("String".to_string()),
                "Vec2" => Some("Vec2".to_string()),
                "Vec3" => Some("Vec3".to_string()),
                "Vec4" => Some("Vec4".to_string()),
                "EntityId" => Some("Entity".to_string()),
                "Vec" => infer_list_value_type_path(&segment.arguments),
                _ => None,
            }
        }
        _ => None,
    }
}

/// 从恰有一个类型实参的 `Vec<T>` 路径推导列表值类型。
fn infer_list_value_type_path(arguments: &PathArguments) -> Option<String> {
    let PathArguments::AngleBracketed(arguments) = arguments else {
        return None;
    };
    if arguments.args.len() != 1 {
        return None;
    }
    let GenericArgument::Type(element_type) = arguments.args.first()? else {
        return None;
    };
    infer_value_type_path(element_type).map(|element| format!("List<{element}>"))
}

/// 根据值类型路径选择默认编辑器提示；无法识别的 schema 类型回退为 `None`。
fn inferred_editor_hint(value_type_path: &str) -> &'static str {
    match value_type_path {
        "Bool" => "Bool",
        "Integer" => "Integer",
        "Unsigned" => "Unsigned",
        "Scalar" => "Scalar",
        "String" => "String",
        "Vec2" => "Vec2",
        "Vec3" => "Vec3",
        "Vec4" => "Vec4",
        "Enum" => "Enum",
        "Entity" => "Entity",
        "Resource" => "Resource",
        "Json" => "Json",
        _ => "None",
    }
}

/// 验证提示名属于 runtime-interface 的 `ReflectEditorHint` 变体集合并生成同名标识符。
///
/// `span` 用作不支持提示名的编译错误位置；名称受支持时返回可插入 token 的 `syn::Ident`。
fn parse_editor_hint(name: &str, span: proc_macro2::Span) -> syn::Result<syn::Ident> {
    const SUPPORTED: &[&str] = &[
        "None",
        "String",
        "MultilineString",
        "Bool",
        "Integer",
        "Unsigned",
        "Scalar",
        "Vec2",
        "Vec3",
        "Vec4",
        "Enum",
        "Entity",
        "Resource",
        "Color",
        "Json",
    ];
    if !SUPPORTED.contains(&name) {
        return Err(syn::Error::new(
            span,
            format!("unsupported reflection editor_hint `{name}`"),
        ));
    }
    Ok(syn::Ident::new(name, span))
}

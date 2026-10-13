//! 将 `zr_reflect` 容器属性和字段属性解析为派生阶段使用的配置值。
//! 本模块只做语法与局部约束检查；字段类型和注册元数据由字段展开及运行时目录继续校验。

use syn::spanned::Spanned;
use syn::{Attribute, LitBool, LitStr, Path};

/// 派生注册写入的序列化策略；具体枚举项映射由 `crate::derive::serialization_tokens` 完成。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SerializationKind {
    /// 不选用序列化策略；未显式设置类型级 `serializable` 时，默认标记为关闭。
    None,
    /// 选择 `Value` 策略，也是当前默认值。
    Value,
    /// 选择 JSON 序列化策略。
    Json,
    /// 选择资源句柄序列化策略。
    ResourceHandle,
    /// 选择实体引用序列化策略。
    EntityReference,
}

impl Default for SerializationKind {
    fn default() -> Self {
        // 容器未显式指定时，派生主流程使用 Value 策略作为缺省值。
        Self::Value
    }
}

/// 反射类型向脚本暴露的级别；默认值由枚举派生为 Private。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ScriptVisibilityKind {
    /// 不对脚本公开。
    #[default]
    Private,
    /// 对脚本公开。
    Public,
}

/// 一个类型的已解析 `zr_reflect(...)` 配置，在单次编译期展开中由派生主流程消费。
///
/// 配置由 [`parse_container_attributes`] 创建，随后传给 `crate::derive::derive_zr_reflect_impl`；
/// 它不进入运行时注册对象。虚拟字段配置再交给 `crate::fields::collect_virtual_fields` 归一化为字段记录。
#[derive(Default)]
pub(crate) struct ContainerAttributes {
    /// 将类型标记为 ECS 组件；与 `resource` 同时出现会被拒绝。
    pub(crate) component: bool,
    /// 将类型标记为资源；与 `component` 同时出现会被拒绝。
    pub(crate) resource: bool,
    /// 覆盖由模块路径和类型名构成的运行时类型路径。
    pub(crate) type_path: Option<String>,
    /// 覆盖字段稳定 ID 的类型身份键；未提供时派生主流程使用 `type_path`。
    pub(crate) identity: Option<String>,
    /// 覆盖运行时注册中显示给工具的类型名称。
    pub(crate) display_name: Option<String>,
    /// 覆盖类型级可序列化标记；缺省时根据序列化策略推导。
    pub(crate) serializable: Option<bool>,
    /// 覆盖类型级编辑器可见标记；缺省为可见。
    pub(crate) editor_visible: Option<bool>,
    /// 覆盖类型级远程可见标记；缺省为不可见。
    pub(crate) remote_visible: Option<bool>,
    /// 覆盖脚本可见级别；缺省为私有。
    pub(crate) script_visibility: ScriptVisibilityKind,
    /// 选择类型级序列化策略；缺省为 `Value`。
    pub(crate) serialization: SerializationKind,
    /// 按声明顺序追加到实体字段后的虚拟反射字段。
    pub(crate) virtual_fields: Vec<FieldAttributes>,
}

/// 一个实体或虚拟字段的已解析 `zr_reflect(...)` 配置，在当前 derive 展开期间传递。
///
/// `parse_field_attributes` 解析实体字段属性，`parse_field_meta` 也解析容器内的虚拟字段；
/// 两者的结果由字段收集器过滤或归一化，再由 token 生成器消费，不作为运行时数据保存。
#[derive(Default)]
pub(crate) struct FieldAttributes {
    /// 从反射字段列表和生成的按名/按槽位分派中排除此实体字段。
    pub(crate) skip: bool,
    /// 覆盖字段在反射元数据和按名分派中使用的名称。
    pub(crate) name: Option<String>,
    /// 覆盖生成稳定字段 ID 时使用的字段身份键；缺省为反射字段名。
    pub(crate) identity: Option<String>,
    /// 写入反射字段元数据的旧名或兼容别名。
    pub(crate) aliases: Vec<String>,
    /// 覆盖字段的反射值类型路径；未知 Rust 类型须显式提供。
    pub(crate) value_type_path: Option<String>,
    /// 覆盖编辑器提示；缺省时根据反射值类型路径推导。
    pub(crate) editor_hint: Option<String>,
    /// 禁止通过生成的反射写入口修改字段。
    pub(crate) readonly: bool,
    /// 覆盖字段级可序列化标记；缺省为可序列化。
    pub(crate) serializable: Option<bool>,
    /// 覆盖字段级编辑器可见标记；缺省为可见。
    pub(crate) editor_visible: Option<bool>,
    /// 提供自定义读访问器；生成分支以 `self` 调用此路径。
    pub(crate) read: Option<Path>,
    /// 提供自定义写访问器；生成分支以 `self` 和反射值调用此路径。
    pub(crate) write: Option<Path>,
}

// 宏展开期先排除角色冲突和缺少访问器的虚拟字段；目录接纳再校验完整注册元数据。
/// 解析容器属性，并在生成 token 前拒绝互斥角色及缺少必要虚拟字段配置的输入。
///
/// # 使用时机与生命周期
/// `crate::derive::derive_zr_reflect_impl` 在选择类型形状和构造注册 token 前调用此函数；返回值仅在本次展开中使用。
///
/// # 参数
/// `attributes` 是目标项的全部 Rust 属性；非 `zr_reflect` 属性会被忽略。
///
/// # 返回值
/// 返回合并后的容器配置；未知键、无效字面量、互斥角色或缺少虚拟字段访问器时返回 `syn::Error`，
/// 再由派生入口转换为编译错误 token。
///
/// # 约束
/// `identity` 必须非空且已去除首尾空白；虚拟字段必须提供名称、值类型路径和读访问器，可写字段还须提供写访问器。
/// 完整字段文本及注册契约由运行时类型目录继续检查。
///
/// # 参考
/// 展开调用点为 [`crate::derive::derive_zr_reflect_impl`]；虚拟字段生成由 [`crate::fields::collect_virtual_fields`] 消费配置。
pub(crate) fn parse_container_attributes(
    attributes: &[Attribute],
) -> syn::Result<ContainerAttributes> {
    let mut parsed = ContainerAttributes::default();
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("zr_reflect"))
    {
        // 多个同名容器属性都累积到同一配置，再统一检查跨键约束。
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("component") {
                parsed.component = true;
            } else if meta.path.is_ident("resource") {
                parsed.resource = true;
            } else if meta.path.is_ident("type_path") {
                parsed.type_path = Some(parse_string(meta)?);
            } else if meta.path.is_ident("identity") {
                parsed.identity = Some(parse_string(meta)?);
            } else if meta.path.is_ident("display_name") {
                parsed.display_name = Some(parse_string(meta)?);
            } else if meta.path.is_ident("serializable") {
                parsed.serializable = Some(parse_bool(meta)?);
            } else if meta.path.is_ident("editor_visible") {
                parsed.editor_visible = Some(parse_bool(meta)?);
            } else if meta.path.is_ident("remote_visible") {
                parsed.remote_visible = Some(parse_bool(meta)?);
            } else if meta.path.is_ident("script_visibility") {
                let span = meta.path.span();
                parsed.script_visibility = match parse_string(meta)?.as_str() {
                    "public" => ScriptVisibilityKind::Public,
                    "private" => ScriptVisibilityKind::Private,
                    other => {
                        return Err(syn::Error::new(
                            span,
                            format!(
                                "unsupported script_visibility `{other}`; expected `public` or `private`"
                            ),
                        ))
                    }
                };
            } else if meta.path.is_ident("serialization") {
                parsed.serialization = parse_serialization(meta)?;
            } else if meta.path.is_ident("field") {
                let mut field = FieldAttributes::default();
                meta.parse_nested_meta(|nested| parse_field_meta(&mut field, nested))?;
                validate_field_attributes(&field)?;
                // TODO: [CR-REFLECT-0001] 确认虚拟字段空名称或值路径是否应在宏期拒绝；当前只查 Some，下一步补充展开与注册测试。
                // 宏期只检查配置项是否存在；空名称由 schema admission 检查，空值类型路径由运行时类型解析检查，
                // 本批 derive 用例未覆盖这两种输入。确认期望报错时机后补充展开与注册测试。
                if field.name.is_none() || field.value_type_path.is_none() || field.read.is_none() {
                    return Err(meta.error(
                        "virtual reflected fields require name, value_type_path, and read",
                    ));
                }
                if !field.readonly && field.write.is_none() {
                    return Err(meta.error(
                        "editable virtual reflected fields require a write accessor",
                    ));
                }
                parsed.virtual_fields.push(field);
            } else {
                return Err(meta.error("unknown zr_reflect type attribute"));
            }
            Ok(())
        })?;
    }

    if parsed.component && parsed.resource {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "a reflected type cannot be both a component and a resource",
        ));
    }
    if parsed
        .type_path
        .as_ref()
        .is_some_and(|path| path.trim().is_empty())
    {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "zr_reflect type_path must not be empty",
        ));
    }
    if parsed
        .identity
        .as_ref()
        .is_some_and(|identity| identity.is_empty() || identity.trim() != identity)
    {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "zr_reflect identity must be non-empty and already trimmed",
        ));
    }

    Ok(parsed)
}

/// 解析实体字段上的全部 `zr_reflect(...)` 属性，并检查只读与身份键约束。
///
/// # 使用时机与生命周期
/// [`crate::fields::collect_fields`] 对每个实体字段调用；结果随字段归一化使用，不进入运行时状态。
///
/// # 参数
/// `attributes` 是单个 Rust 字段的属性集合；其他属性由本函数忽略。
///
/// # 返回值
/// 返回合并后的字段配置；未知键、错误字面量、只读字段附带写访问器或无效身份键会产生 `syn::Error`。
///
/// # 约束
/// `identity` 必须非空且已去除首尾空白。虚拟字段的必填项由 [`parse_container_attributes`] 额外检查。
///
/// # 参考
/// 容器虚拟字段与实体字段共用 [`parse_field_meta`]。
pub(crate) fn parse_field_attributes(attributes: &[Attribute]) -> syn::Result<FieldAttributes> {
    let mut parsed = FieldAttributes::default();
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("zr_reflect"))
    {
        attribute.parse_nested_meta(|meta| parse_field_meta(&mut parsed, meta))?;
    }

    validate_field_attributes(&parsed)?;
    Ok(parsed)
}

/// 将一个嵌套键应用到字段配置；本函数只更新单键状态，组合约束交由解析完成后的校验处理。
///
/// 容器中的 `field(...)` 与实体字段自身的 `zr_reflect(...)` 共用此解析器，因此两种字段声明接受相同键集。
fn parse_field_meta(
    parsed: &mut FieldAttributes,
    meta: syn::meta::ParseNestedMeta<'_>,
) -> syn::Result<()> {
    if meta.path.is_ident("skip") {
        parsed.skip = true;
    } else if meta.path.is_ident("name") {
        parsed.name = Some(parse_string(meta)?);
    } else if meta.path.is_ident("identity") {
        parsed.identity = Some(parse_string(meta)?);
    } else if meta.path.is_ident("alias") {
        parsed.aliases.push(parse_string(meta)?);
    } else if meta.path.is_ident("value_type_path") {
        parsed.value_type_path = Some(parse_string(meta)?);
    } else if meta.path.is_ident("editor_hint") {
        parsed.editor_hint = Some(parse_string(meta)?);
    } else if meta.path.is_ident("readonly") {
        parsed.readonly = true;
    } else if meta.path.is_ident("serializable") {
        parsed.serializable = Some(parse_bool(meta)?);
    } else if meta.path.is_ident("editor_visible") {
        parsed.editor_visible = Some(parse_bool(meta)?);
    } else if meta.path.is_ident("read") {
        parsed.read = Some(parse_path(meta)?);
    } else if meta.path.is_ident("write") {
        parsed.write = Some(parse_path(meta)?);
    } else {
        return Err(meta.error("unknown zr_reflect field attribute"));
    }
    Ok(())
}

/// 检查字段键之间的互斥关系，并要求稳定身份键非空且无首尾空白。
fn validate_field_attributes(parsed: &FieldAttributes) -> syn::Result<()> {
    if parsed.readonly && parsed.write.is_some() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "a readonly reflected field cannot declare a write accessor",
        ));
    }
    if parsed
        .identity
        .as_ref()
        .is_some_and(|identity| identity.is_empty() || identity.trim() != identity)
    {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "a reflected field identity must be non-empty and already trimmed",
        ));
    }
    Ok(())
}

/// 读取字符串型嵌套属性并返回其解码后的 Rust 字符串值。
fn parse_string(meta: syn::meta::ParseNestedMeta<'_>) -> syn::Result<String> {
    Ok(meta.value()?.parse::<LitStr>()?.value())
}

/// 读取布尔型嵌套属性；调用者负责将返回值放入对应的可选配置项。
fn parse_bool(meta: syn::meta::ParseNestedMeta<'_>) -> syn::Result<bool> {
    Ok(meta.value()?.parse::<LitBool>()?.value())
}

/// 将字符串字面量解析为 Rust 路径，供生成的自定义访问分支引用。
fn parse_path(meta: syn::meta::ParseNestedMeta<'_>) -> syn::Result<Path> {
    let value = meta.value()?.parse::<LitStr>()?;
    value.parse()
}

/// 将策略字符串限定到五个受支持的序列化变体。
fn parse_serialization(meta: syn::meta::ParseNestedMeta<'_>) -> syn::Result<SerializationKind> {
    let value = parse_string(meta)?;
    match value.as_str() {
        "none" => Ok(SerializationKind::None),
        "value" => Ok(SerializationKind::Value),
        "json" => Ok(SerializationKind::Json),
        "resource_handle" => Ok(SerializationKind::ResourceHandle),
        "entity_reference" => Ok(SerializationKind::EntityReference),
        other => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("unsupported reflection serialization `{other}`"),
        )),
    }
}

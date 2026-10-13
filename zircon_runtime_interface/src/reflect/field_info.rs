use serde::{Deserialize, Serialize};

use super::{
    ReflectEditorHint, ReflectEnumOption, ReflectFieldId, ReflectNumericRange, ReflectedValue,
};

/// 稳定 ID 用于访问，name 用于诊断及显式旧名导入，display_name 与 documentation 供编辑器呈现。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectFieldInfo {
    /// 独立于当前名称和目录槽位的稳定字段身份。
    pub id: ReflectFieldId,
    /// 当前 schema 字段名；用于诊断、显示与别名解析，不作为长期身份。
    pub name: String,
    /// 面向编辑器的显示名称。
    pub display_name: String,
    /// 显式兼容名称；运行时目录会校验并限定其解析范围。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// 与 `ReflectedValue` 相配的 schema 值类型标识；Runtime 注册端进一步解析并校验。
    pub value_type_path: String,
    /// 该字段是否允许反射写入口修改。
    pub editable: bool,
    /// 该字段是否参与序列化投影。
    pub serializable: bool,
    /// 该字段是否可出现在编辑器 schema 查询中。
    pub editor_visible: bool,
    /// 可选默认值；Runtime 注册端会验证其预算与声明类型兼容性。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<ReflectedValue>,
    /// 可选编辑数值范围；数值合法性由本类型构造器校验，字段类型适配由 Runtime 注册端校验。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_range: Option<ReflectNumericRange>,
    /// 枚举值及其呈现信息；Runtime 注册端会校验值唯一性和数量。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_options: Vec<ReflectEnumOption>,
    /// 编辑控件提示；与 `value_type_path` 的兼容性由 Runtime 注册时复核。
    pub editor_hint: ReflectEditorHint,
    /// 可选字段说明，供工具和编辑器呈现。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
}

impl ReflectFieldInfo {
    /// 创建字段元数据，并采用可编辑、可序列化、编辑器可见及无可选扩展值的默认项。
    pub fn new(
        id: ReflectFieldId,
        name: impl Into<String>,
        value_type_path: impl Into<String>,
        editor_hint: ReflectEditorHint,
    ) -> Self {
        let name = name.into();
        Self {
            id,
            display_name: name.clone(),
            name,
            aliases: Vec::new(),
            value_type_path: value_type_path.into(),
            editable: true,
            serializable: true,
            editor_visible: true,
            default_value: None,
            numeric_range: None,
            enum_options: Vec::new(),
            editor_hint,
            documentation: None,
        }
    }

    /// 先从稳定身份键计算 ID，再创建与 [`Self::new`] 相同默认值的字段元数据。
    pub fn from_stable_keys(
        owner_key: &str,
        field_key: &str,
        name: impl Into<String>,
        value_type_path: impl Into<String>,
        editor_hint: ReflectEditorHint,
    ) -> Self {
        Self::new(
            ReflectFieldId::from_stable_keys(owner_key, field_key),
            name,
            value_type_path,
            editor_hint,
        )
    }

    /// 覆盖编辑器呈现名称；不改变字段名或稳定 ID。
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    /// 设置供目录按字段作用域解析的兼容别名。
    pub fn with_aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases = aliases;
        self
    }

    /// 设置反射写入是否允许修改该字段。
    pub fn with_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// 设置该字段是否参与序列化投影。
    pub fn with_serializable(mut self, serializable: bool) -> Self {
        self.serializable = serializable;
        self
    }

    /// 设置编辑器 schema 查询是否可返回该字段。
    pub fn with_editor_visible(mut self, editor_visible: bool) -> Self {
        self.editor_visible = editor_visible;
        self
    }

    /// 设置默认值；调用者仍须在 Runtime 注册前满足声明值类型及值预算。
    pub fn with_default_value(mut self, default_value: ReflectedValue) -> Self {
        self.default_value = Some(default_value);
        self
    }

    /// 附加已构造的数值范围元数据。
    pub fn with_numeric_range(mut self, numeric_range: ReflectNumericRange) -> Self {
        self.numeric_range = Some(numeric_range);
        self
    }

    /// 附加编辑器枚举选项；Runtime 注册端检查选项与字段声明的兼容性。
    pub fn with_enum_options(mut self, enum_options: Vec<ReflectEnumOption>) -> Self {
        self.enum_options = enum_options;
        self
    }

    /// 附加供工具呈现的字段说明。
    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }
}

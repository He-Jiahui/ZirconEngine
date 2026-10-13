use serde::{Deserialize, Serialize};

/// 编辑器选择控件的提示；它不替代运行时对字段值类型及提示兼容性的注册校验。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectEditorHint {
    /// 不指定专用控件。
    None,
    /// 单行文本。
    String,
    /// 多行文本。
    MultilineString,
    /// 布尔开关。
    Bool,
    /// 有符号整数。
    Integer,
    /// 无符号整数。
    Unsigned,
    /// 标量浮点数。
    Scalar,
    /// 二维向量。
    Vec2,
    /// 三维向量。
    Vec3,
    /// 四分量向量或四元数的编辑控件。
    Vec4,
    /// 枚举选项控件；选项内容另由字段元数据提供。
    Enum,
    /// 实体引用控件。
    Entity,
    /// 资源引用控件。
    Resource,
    /// 颜色控件。
    Color,
    /// 通用 JSON 编辑控件。
    Json,
}

/// 一个枚举字段可展示的稳定值、用户可见名称及可选说明。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectEnumOption {
    /// 读写请求中实际使用的枚举值。
    pub value: String,
    /// 编辑器展示的标签。
    pub display_name: String,
    /// 展示给作者的选项说明。
    pub documentation: Option<String>,
}

impl ReflectEnumOption {
    /// 创建无说明的枚举选项；目录注册端会检查值与标签的规范性及选项唯一性。
    pub fn new(value: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            display_name: display_name.into(),
            documentation: None,
        }
    }

    /// 附加选项说明并返回更新后的元数据。
    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }
}

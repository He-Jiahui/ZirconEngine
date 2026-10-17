//! 组件数据源及属性行的自有描述符，供宿主交换可绑定字段和属性编辑提示。
//! 构建器记录元数据；目标解析、数值约束执行与数据写入由宿主负责。

use serde::{Deserialize, Serialize};

use crate::ui::component::{UiValidationLevel, UiValueKind};

/// Describes an editor/runtime data source that component events can target.
///
/// # 使用与生命周期
/// 宿主枚举数据源时创建并传递此值；字符串、字段列表和可接纳值种类均由描述符拥有。
/// 主体与路径前缀是供域解释的元数据，不持有对应实体或资源，也不授予实际写入权限。
/// 只读工厂默认值可用值接收构建器修改；它们与反序列化字段的缺省规则分别定义。
///
/// # 参考
/// [`Self::new`]、[`UiComponentDataSourceFieldDescriptor`]。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentDataSourceDescriptor {
    pub domain: String,
    pub source_name: String,
    pub display_name: String,
    pub kind: UiComponentDataSourceKind,
    pub subject: Option<String>,
    pub path_prefix: Option<String>,
    pub writable: bool,
    pub value_kinds: Vec<UiValueKind>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<UiComponentDataSourceFieldDescriptor>,
}

impl UiComponentDataSourceDescriptor {
    /// 创建尚未指定主体、前缀、值种类及字段列表的只读描述符。
    ///
    /// # 参数与返回
    /// `domain` 和 `source_name` 标识宿主域及域内来源，`display_name` 用于展示，`kind` 分类来源。
    /// 字符串转换为自有值；返回值不查询真实来源，可用后续构建器补充可写位和字段元数据。
    pub fn new(
        domain: impl Into<String>,
        source_name: impl Into<String>,
        display_name: impl Into<String>,
        kind: UiComponentDataSourceKind,
    ) -> Self {
        Self {
            domain: domain.into(),
            source_name: source_name.into(),
            display_name: display_name.into(),
            kind,
            subject: None,
            path_prefix: None,
            writable: false,
            value_kinds: Vec::new(),
            fields: Vec::new(),
        }
    }

    /// 接管主体标识并替换旧值，返回描述符；不解析或保留主体对象。
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// 接管域内路径前缀并替换旧值，返回描述符；不拼接、规范化或验证路径。
    pub fn with_path_prefix(mut self, path_prefix: impl Into<String>) -> Self {
        self.path_prefix = Some(path_prefix.into());
        self
    }

    /// 仅替换声明的可写位并返回描述符；消费端仍负责实际权限和写入校验。
    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    /// 消费迭代器并替换全部值种类，返回描述符；按输入顺序收集，不去重或校验组合。
    pub fn with_value_kinds(mut self, value_kinds: impl IntoIterator<Item = UiValueKind>) -> Self {
        self.value_kinds = value_kinds.into_iter().collect();
        self
    }

    /// 接管并追加一个字段，返回描述符；不按字段路径合并重复项。
    pub fn with_field(mut self, field: UiComponentDataSourceFieldDescriptor) -> Self {
        self.fields.push(field);
        self
    }

    /// 消费迭代器并替换整个字段列表，返回描述符；与 [`Self::with_field`] 的追加行为不同。
    pub fn with_fields(
        mut self,
        fields: impl IntoIterator<Item = UiComponentDataSourceFieldDescriptor>,
    ) -> Self {
        self.fields = fields.into_iter().collect();
        self
    }
}

/// 数据源的分类标签；自定义变体拥有其标签文字，类型本身不注册或路由适配器。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiComponentDataSourceKind {
    Inspector,
    Reflection,
    AssetEditor,
    Showcase,
    Custom(String),
}

/// Describes one bindable property row resolved from a data source.
///
/// # 约束与生命周期
/// 描述符拥有路径、展示文字、选项及校验提示；不借用真实属性值。
/// 范围、步长、集合元素种类及引用类别都只作元数据记录，不据此改变值种类或校验输入。
/// 多类提示可同时存在，构造与反序列化不在此处建立它们之间的业务一致性。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentDataSourceFieldDescriptor {
    pub path: String,
    pub display_name: String,
    pub value_kind: UiValueKind,
    pub writable: bool,
    pub group: Option<String>,
    pub collapsed: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub options: Vec<UiComponentDataSourceFieldOption>,
    pub element_kind: Option<UiValueKind>,
    pub key_kind: Option<UiValueKind>,
    pub map_value_kind: Option<UiValueKind>,
    pub reference_kind: Option<String>,
    pub validation_level: Option<UiValidationLevel>,
    pub validation_message: Option<String>,
}

impl UiComponentDataSourceFieldDescriptor {
    /// 创建没有附加编辑提示的只读字段描述符。
    ///
    /// # 参数与返回
    /// `path` 为域内字段路径，`display_name` 为展示文字，`value_kind` 为声明值种类。
    /// 字符串被拥有；返回值的可写位和折叠位关闭，选项为空，所有可选提示为空。
    /// 该工厂不查找真实字段，后续值接收构建器只更新提示并返回自有描述符。
    pub fn new(
        path: impl Into<String>,
        display_name: impl Into<String>,
        value_kind: UiValueKind,
    ) -> Self {
        Self {
            path: path.into(),
            display_name: display_name.into(),
            value_kind,
            writable: false,
            group: None,
            collapsed: false,
            min: None,
            max: None,
            step: None,
            options: Vec::new(),
            element_kind: None,
            key_kind: None,
            map_value_kind: None,
            reference_kind: None,
            validation_level: None,
            validation_message: None,
        }
    }

    /// 替换字段声明的可写位并返回描述符；不执行宿主权限检查。
    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    /// 接管分组文字并替换旧值，返回描述符；不移动或重新排列实际属性。
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// 替换初始折叠提示并返回描述符；不直接改变已显示的属性面板。
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// 保存数值范围提示并返回描述符；不检查有限性、上下界顺序，也不截断真实值。
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }

    /// 保存数值步长提示并返回描述符；不检查其符号或有限性，也不执行量化。
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// 消费迭代器并替换选项列表，返回描述符；保持输入顺序，不检查标识唯一性。
    pub fn options(
        mut self,
        options: impl IntoIterator<Item = UiComponentDataSourceFieldOption>,
    ) -> Self {
        self.options = options.into_iter().collect();
        self
    }

    /// 替换数组元素种类提示并返回描述符；不修改或检查本字段的声明值种类。
    pub fn array_element_kind(mut self, element_kind: UiValueKind) -> Self {
        self.element_kind = Some(element_kind);
        self
    }

    /// 同时替换映射键、值的种类提示并返回描述符；不创建映射或校验组合。
    pub fn map_kinds(mut self, key_kind: UiValueKind, value_kind: UiValueKind) -> Self {
        self.key_kind = Some(key_kind);
        self.map_value_kind = Some(value_kind);
        self
    }

    /// 接管引用类别文字并替换旧值，返回描述符；不加载或验证被引用对象。
    pub fn reference_kind(mut self, reference_kind: impl Into<String>) -> Self {
        self.reference_kind = Some(reference_kind.into());
        self
    }

    /// 同时保存校验级别和自有提示文字，返回描述符；不在此函数中运行校验。
    pub fn validation(mut self, level: UiValidationLevel, message: impl Into<String>) -> Self {
        self.validation_level = Some(level);
        self.validation_message = Some(message.into());
        self
    }
}

/// 属性编辑候选项的自有标识、展示名和禁用提示；不执行选中或禁用行为。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiComponentDataSourceFieldOption {
    pub id: String,
    pub display_name: String,
    pub disabled: bool,
}

impl UiComponentDataSourceFieldOption {
    /// 接管候选项标识和展示名，返回默认未禁用的选项；不校验标识是否为空或重复。
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            disabled: false,
        }
    }

    /// 替换禁用提示并返回自有选项；具体控件决定如何应用此提示。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

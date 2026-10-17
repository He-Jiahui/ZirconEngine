//! 组件事件目标只携带域、主体和字段路径；具体解析与写入由宿主适配器负责。

use serde::{Deserialize, Serialize};

/// Identifies the runtime-owned domain and field path a component event targets.
///
/// # 约束与生命周期
/// 域、可选主体和字段路径均为自有字符串，可脱离事件生产者持有。
/// 工厂只填入约定值；字符串、主体存在性和路径语义由具体宿主解释及校验。
/// 派生比较和哈希使用原始字段值，不执行路径规范化或域名解析。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiComponentBindingTarget {
    /// 路由域由宿主适配器解释，未知域必须由消费端拒绝。
    pub domain: String,
    /// 宿主用来选择具体数据主体的标识；为空时由对应域解释其含义。
    pub subject: Option<String>,
    /// 域内的字段或控件路径；本值不拆分或校验路径。
    pub path: String,
}

impl UiComponentBindingTarget {
    /// 创建尚未指定主体的目标。
    ///
    /// # 参数与返回
    /// `domain` 指定交由哪个宿主域解释，`path` 为该域内的目标路径；二者均转换为自有字符串。
    /// 返回主体为空的独立值，不查找文档、实体或控件；可用 [`Self::with_subject`] 补充主体。
    pub fn new(domain: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            subject: None,
            path: path.into(),
        }
    }

    /// 将参数转换为自有主体标识，替换已有主体并返回目标；不验证该主体是否存在。
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// 以主体和字段路径创建检查器域目标；参数被拥有，解析与写入由检查器宿主负责。
    pub fn inspector(subject: impl Into<String>, field_path: impl Into<String>) -> Self {
        Self::new("inspector", field_path).with_subject(subject)
    }

    /// 以主体和字段路径创建反射域目标；构造时不解析反射字段或验证主体。
    pub fn reflection(subject: impl Into<String>, field_path: impl Into<String>) -> Self {
        Self::new("reflection", field_path).with_subject(subject)
    }

    /// 以主体和字段路径创建资源编辑域目标；构造时不打开资源或执行编辑。
    pub fn asset_editor(subject: impl Into<String>, field_path: impl Into<String>) -> Self {
        Self::new("asset_editor", field_path).with_subject(subject)
    }

    /// 以控件标识作为路径创建展示域目标，主体为空；结果由对应展示宿主解释。
    pub fn showcase(control_id: impl Into<String>) -> Self {
        Self::new("showcase", control_id)
    }
}

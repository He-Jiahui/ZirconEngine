//! 把可翻译的语义消息与不可翻译的外部日志/旧归档文本分开保存。
//! 配置保存编号和参数，视图投影按当前语言渲染；英文展示实现主要服务诊断和既有比较。

use serde::{Deserialize, Serialize, Serializer};

use crate::settings::HubLanguage;

use super::HubMessageId;

/// 状态/历史的语言无关载体；结构化分支保留编号和参数，原文分支保留外部诊断或旧记录。
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "HubMessageRepr")]
pub enum HubMessage {
    Structured {
        id: HubMessageId,
        params: Vec<String>,
    },
    RawText(String),
}

impl HubMessage {
    /// 构造无参数的语义消息；带参数模板应使用参数构造入口。
    pub fn new(id: HubMessageId) -> Self {
        Self::Structured {
            id,
            params: Vec::new(),
        }
    }

    // TODO: [CR-HUBSTATE-0002] 确认已知编号参数数量的加载/构造约束；当前不校验且缺少异常参数用例，需检查归档兼容测试。
    /// 构造待显示的语义消息；调用端应满足编号的参数数量与顺序约定。
    /// 参数应是未经本地化的路径、目标名或诊断数据，正文在显示时选择语言。
    pub fn with_params<P: Into<String>>(
        id: HubMessageId,
        params: impl IntoIterator<Item = P>,
    ) -> Self {
        Self::Structured {
            id,
            params: params.into_iter().map(Into::into).collect(),
        }
    }

    /// 保存外部错误、日志或无法识别的归档消息；这段原文不会随语言切换。
    pub fn raw_text(text: impl Into<String>) -> Self {
        Self::RawText(text.into())
    }

    /// 为可选诊断和旧配置字段提供空原文；它表示没有内容而非某个翻译编号。
    pub fn empty() -> Self {
        Self::RawText(String::new())
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::RawText(text) => text.trim().is_empty(),
            Self::Structured { .. } => false,
        }
    }

    /// 按英文渲染结果做诊断/兼容查找；交互界面应按当前语言渲染。
    pub fn contains(&self, needle: &str) -> bool {
        self.render(HubLanguage::English).contains(needle)
    }

    /// 由语言投影在显示时调用；语义记录可在切换语言后重新呈现，原文保持原样。
    pub fn render(&self, language: HubLanguage) -> String {
        match self {
            Self::RawText(text) => text.clone(),
            Self::Structured { id, params } => render_template(id.template(language), params),
        }
    }

    /// 组合状态与恢复提示；没有主消息时仍可显示有效恢复建议。
    pub fn render_with_recovery(&self, recovery: Option<&Self>, language: HubLanguage) -> String {
        match recovery.filter(|message| !message.is_empty()) {
            Some(recovery) if self.is_empty() => recovery.render(language),
            Some(recovery) => format!("{} - {}", self.render(language), recovery.render(language)),
            None => self.render(language),
        }
    }
}

impl Default for HubMessage {
    fn default() -> Self {
        Self::empty()
    }
}

impl PartialEq<&str> for HubMessage {
    fn eq(&self, other: &&str) -> bool {
        self.render(HubLanguage::English) == *other
    }
}

impl PartialEq<String> for HubMessage {
    fn eq(&self, other: &String) -> bool {
        self.render(HubLanguage::English) == *other
    }
}

// 通用格式化沿用英文诊断口径；当前界面语言由显式渲染入口控制。
impl std::fmt::Display for HubMessage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.render(HubLanguage::English))
    }
}

/// 配置读取兼容旧字符串与结构化记录；未知编号保留可读内容，避免丢弃整个配置。
#[derive(Deserialize)]
#[serde(untagged)]
enum HubMessageRepr {
    Structured {
        id: String,
        #[serde(default)]
        params: Vec<String>,
    },
    ArchivedRawText(String),
}

impl From<HubMessageRepr> for HubMessage {
    fn from(repr: HubMessageRepr) -> Self {
        match repr {
            HubMessageRepr::ArchivedRawText(text) => Self::RawText(text),
            HubMessageRepr::Structured { id, params } => match HubMessageId::from_str_id(&id) {
                Some(id) => Self::Structured { id, params },
                None if params.is_empty() => Self::RawText(id),
                None => Self::RawText(format!("{id}: {}", params.join(", "))),
            },
        }
    }
}

impl Serialize for HubMessage {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        match self {
            Self::RawText(text) => serializer.serialize_str(text),
            Self::Structured { id, params } => {
                let mut row = serializer.serialize_struct("HubMessage", 2)?;
                row.serialize_field("id", id.as_str())?;
                row.serialize_field("params", params)?;
                row.end()
            }
        }
    }
}

// BUG: [CR-HUBSTATE-0001] 较早参数包含后续占位符时会被再次替换，改变原始路径/诊断；证据：项目记录失败消息同时插入目录和错误文本。
// 模板替换属于显示边界，用户参数应作为不可再解释的原文保留。
fn render_template(template: &str, params: &[String]) -> String {
    let mut rendered = template.to_string();
    for (index, param) in params.iter().enumerate() {
        rendered = rendered.replace(&format!("{{{index}}}"), param);
    }
    rendered
}

#[cfg(test)]
#[path = "tests/message.rs"]
mod tests;

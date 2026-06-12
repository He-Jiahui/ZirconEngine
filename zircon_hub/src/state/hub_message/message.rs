use serde::{Deserialize, Serialize, Serializer};

use crate::settings::HubLanguage;

use super::HubMessageId;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "HubMessageRepr")]
pub enum HubMessage {
    Structured {
        id: HubMessageId,
        params: Vec<String>,
    },
    Legacy(String),
}

impl HubMessage {
    pub fn new(id: HubMessageId) -> Self {
        Self::Structured {
            id,
            params: Vec::new(),
        }
    }

    pub fn with_params<P: Into<String>>(
        id: HubMessageId,
        params: impl IntoIterator<Item = P>,
    ) -> Self {
        Self::Structured {
            id,
            params: params.into_iter().map(Into::into).collect(),
        }
    }

    pub fn legacy(text: impl Into<String>) -> Self {
        Self::Legacy(text.into())
    }

    pub fn empty() -> Self {
        Self::Legacy(String::new())
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Legacy(text) => text.trim().is_empty(),
            Self::Structured { .. } => false,
        }
    }

    pub fn contains(&self, needle: &str) -> bool {
        self.render(HubLanguage::English).contains(needle)
    }

    pub fn render(&self, language: HubLanguage) -> String {
        match self {
            Self::Legacy(text) => text.clone(),
            Self::Structured { id, params } => render_template(id.template(language), params),
        }
    }

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

impl std::fmt::Display for HubMessage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.render(HubLanguage::English))
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum HubMessageRepr {
    Structured {
        id: String,
        #[serde(default)]
        params: Vec<String>,
    },
    Legacy(String),
}

impl From<HubMessageRepr> for HubMessage {
    fn from(repr: HubMessageRepr) -> Self {
        match repr {
            HubMessageRepr::Legacy(text) => Self::Legacy(text),
            HubMessageRepr::Structured { id, params } => match HubMessageId::from_str_id(&id) {
                Some(id) => Self::Structured { id, params },
                None if params.is_empty() => Self::Legacy(id),
                None => Self::Legacy(format!("{id}: {}", params.join(", "))),
            },
        }
    }
}

impl Serialize for HubMessage {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        match self {
            Self::Legacy(text) => serializer.serialize_str(text),
            Self::Structured { id, params } => {
                let mut row = serializer.serialize_struct("HubMessage", 2)?;
                row.serialize_field("id", id.as_str())?;
                row.serialize_field("params", params)?;
                row.end()
            }
        }
    }
}

fn render_template(template: &str, params: &[String]) -> String {
    let mut rendered = template.to_string();
    for (index, param) in params.iter().enumerate() {
        rendered = rendered.replace(&format!("{{{index}}}"), param);
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{HubMessage, HubMessageId, ShellMessageId};

    #[test]
    fn legacy_string_deserializes_into_legacy_branch() {
        let message: HubMessage = serde_json::from_str("\"old detail\"").unwrap();

        assert_eq!(message, "old detail");
    }

    #[test]
    fn unknown_id_degrades_to_legacy_instead_of_failing_file_load() {
        let message: HubMessage =
            serde_json::from_str(r#"{"id":"future.message","params":["a","b"]}"#).unwrap();

        assert_eq!(message, "future.message: a, b");
    }

    #[test]
    fn structured_message_renders_template_parameters() {
        let message = HubMessage::with_params(
            HubMessageId::Shell(ShellMessageId::OpenedPath),
            ["C:/Projects/Game"],
        );

        assert_eq!(
            message.render(HubLanguage::Chinese),
            "已打开 C:/Projects/Game"
        );
    }
}

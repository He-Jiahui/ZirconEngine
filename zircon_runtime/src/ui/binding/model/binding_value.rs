use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiBindingValue {
    String(String),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Bool(bool),
    Null,
    Array(Vec<UiBindingValue>),
}

impl UiBindingValue {
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn unsigned(value: u32) -> Self {
        Self::Unsigned(value as u64)
    }

    pub fn array(values: impl Into<Vec<UiBindingValue>>) -> Self {
        Self::Array(values.into())
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Self::Unsigned(value) => (*value).try_into().ok(),
            Self::Signed(value) if *value >= 0 => (*value as u64).try_into().ok(),
            _ => None,
        }
    }

    pub(crate) fn native_repr(&self) -> String {
        match self {
            Self::String(value) => format!("\"{}\"", escape_string(value)),
            Self::Unsigned(value) => value.to_string(),
            Self::Signed(value) => value.to_string(),
            Self::Float(value) => {
                let mut rendered = value.to_string();
                if !rendered.contains('.') && !rendered.contains('e') && !rendered.contains('E') {
                    rendered.push_str(".0");
                }
                rendered
            }
            Self::Bool(value) => value.to_string(),
            Self::Null => "null".to_string(),
            Self::Array(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(Self::native_repr)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

fn escape_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

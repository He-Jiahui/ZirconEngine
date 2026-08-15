use serde::{Deserialize, Serialize};

/// Stable host handle value exposed to script VMs.
///
/// The framework layer owns the neutral value representation, so it stores the
/// numeric handle instead of depending on the concrete VM subsystem handle type.
pub type ScriptHostHandleValue = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptHostValueKind {
    Null,
    Bool,
    Int,
    Float,
    String,
    Bytes,
    HostHandle,
}

impl ScriptHostValueKind {
    pub fn default_zr_type_name(self) -> &'static str {
        match self {
            Self::Null => "void",
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Float => "float",
            Self::String => "string",
            // ZrVM strings are text; retain arbitrary bytes through its typed array boundary.
            Self::Bytes => "container.Array<uint>",
            Self::HostHandle => "int",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptHostTypeRef {
    pub value_kind: ScriptHostValueKind,
    pub type_name: String,
}

impl ScriptHostTypeRef {
    pub fn new(value_kind: ScriptHostValueKind, type_name: impl Into<String>) -> Self {
        Self {
            value_kind,
            type_name: type_name.into(),
        }
    }

    pub fn from_value_kind(value_kind: ScriptHostValueKind) -> Self {
        Self::new(value_kind, value_kind.default_zr_type_name())
    }
}

impl Default for ScriptHostTypeRef {
    fn default() -> Self {
        Self::from_value_kind(ScriptHostValueKind::Null)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptHostPrototypeKind {
    Module,
    Class,
    Interface,
    Struct,
    Enum,
    Native,
}

impl Default for ScriptHostPrototypeKind {
    fn default() -> Self {
        Self::Struct
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ScriptHostValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    HostHandle(ScriptHostHandleValue),
}

impl ScriptHostValue {
    pub fn kind(&self) -> ScriptHostValueKind {
        match self {
            Self::Null => ScriptHostValueKind::Null,
            Self::Bool(_) => ScriptHostValueKind::Bool,
            Self::Int(_) => ScriptHostValueKind::Int,
            Self::Float(_) => ScriptHostValueKind::Float,
            Self::String(_) => ScriptHostValueKind::String,
            Self::Bytes(_) => ScriptHostValueKind::Bytes,
            Self::HostHandle(_) => ScriptHostValueKind::HostHandle,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptHostError {
    pub message: String,
}

impl ScriptHostError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub type ScriptHostResult = Result<ScriptHostValue, ScriptHostError>;

pub trait ScriptHostIntoValue {
    fn script_host_type_ref() -> ScriptHostTypeRef;

    fn into_script_host_value(self) -> ScriptHostValue;
}

impl ScriptHostIntoValue for bool {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Bool)
    }

    fn into_script_host_value(self) -> ScriptHostValue {
        ScriptHostValue::Bool(self)
    }
}

impl ScriptHostIntoValue for i64 {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Int)
    }

    fn into_script_host_value(self) -> ScriptHostValue {
        ScriptHostValue::Int(self)
    }
}

impl ScriptHostIntoValue for f64 {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Float)
    }

    fn into_script_host_value(self) -> ScriptHostValue {
        ScriptHostValue::Float(self)
    }
}

impl ScriptHostIntoValue for f32 {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Float)
    }

    fn into_script_host_value(self) -> ScriptHostValue {
        ScriptHostValue::Float(self as f64)
    }
}

impl ScriptHostIntoValue for String {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::String)
    }

    fn into_script_host_value(self) -> ScriptHostValue {
        ScriptHostValue::String(self)
    }
}

impl ScriptHostIntoValue for Vec<u8> {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Bytes)
    }

    fn into_script_host_value(self) -> ScriptHostValue {
        ScriptHostValue::Bytes(self)
    }
}

impl ScriptHostIntoValue for () {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Null)
    }

    fn into_script_host_value(self) -> ScriptHostValue {
        ScriptHostValue::Null
    }
}

impl ScriptHostIntoValue for ScriptHostValue {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Null)
    }

    fn into_script_host_value(self) -> ScriptHostValue {
        self
    }
}

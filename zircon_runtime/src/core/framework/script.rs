//! Script-facing framework contracts shared by VM backends and host exports.

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
            Self::Bytes => "string",
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

pub trait ScriptHostFromValue: Sized {
    fn script_host_type_ref() -> ScriptHostTypeRef;

    fn from_script_host_value(
        value: &ScriptHostValue,
        argument_index: usize,
    ) -> Result<Self, ScriptHostError>;
}

pub trait ScriptHostIntoValue {
    fn script_host_type_ref() -> ScriptHostTypeRef;

    fn into_script_host_value(self) -> ScriptHostValue;
}

impl ScriptHostFromValue for bool {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Bool)
    }

    fn from_script_host_value(
        value: &ScriptHostValue,
        argument_index: usize,
    ) -> Result<Self, ScriptHostError> {
        match value {
            ScriptHostValue::Bool(value) => Ok(*value),
            value => Err(argument_type_error(
                argument_index,
                ScriptHostValueKind::Bool,
                value,
            )),
        }
    }
}

impl ScriptHostIntoValue for bool {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Bool)
    }

    fn into_script_host_value(self) -> ScriptHostValue {
        ScriptHostValue::Bool(self)
    }
}

impl ScriptHostFromValue for i64 {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Int)
    }

    fn from_script_host_value(
        value: &ScriptHostValue,
        argument_index: usize,
    ) -> Result<Self, ScriptHostError> {
        match value {
            ScriptHostValue::Int(value) => Ok(*value),
            value => Err(argument_type_error(
                argument_index,
                ScriptHostValueKind::Int,
                value,
            )),
        }
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

impl ScriptHostFromValue for f64 {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Float)
    }

    fn from_script_host_value(
        value: &ScriptHostValue,
        argument_index: usize,
    ) -> Result<Self, ScriptHostError> {
        match value {
            ScriptHostValue::Float(value) => Ok(*value),
            ScriptHostValue::Int(value) => Ok(*value as f64),
            value => Err(argument_type_error(
                argument_index,
                ScriptHostValueKind::Float,
                value,
            )),
        }
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

impl ScriptHostFromValue for f32 {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Float)
    }

    fn from_script_host_value(
        value: &ScriptHostValue,
        argument_index: usize,
    ) -> Result<Self, ScriptHostError> {
        f64::from_script_host_value(value, argument_index).map(|value| value as f32)
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

impl ScriptHostFromValue for String {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::String)
    }

    fn from_script_host_value(
        value: &ScriptHostValue,
        argument_index: usize,
    ) -> Result<Self, ScriptHostError> {
        match value {
            ScriptHostValue::String(value) => Ok(value.clone()),
            value => Err(argument_type_error(
                argument_index,
                ScriptHostValueKind::String,
                value,
            )),
        }
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

fn argument_type_error(
    argument_index: usize,
    expected: ScriptHostValueKind,
    actual: &ScriptHostValue,
) -> ScriptHostError {
    ScriptHostError::new(format!(
        "argument {argument_index} expected {:?}, received {:?}",
        expected,
        actual.kind()
    ))
}

pub trait ZirconScriptType {
    fn script_host_type_descriptor() -> ScriptHostTypeDescriptor;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptHostParameterDescriptor {
    pub name: String,
    pub value_kind: ScriptHostValueKind,
    #[serde(default)]
    pub type_ref: ScriptHostTypeRef,
    pub documentation: Option<String>,
}

impl ScriptHostParameterDescriptor {
    pub fn new(name: impl Into<String>, value_kind: ScriptHostValueKind) -> Self {
        Self {
            name: name.into(),
            value_kind,
            type_ref: ScriptHostTypeRef::from_value_kind(value_kind),
            documentation: None,
        }
    }

    pub fn with_type_ref(mut self, type_ref: ScriptHostTypeRef) -> Self {
        self.value_kind = type_ref.value_kind;
        self.type_ref = type_ref;
        self
    }

    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptHostFunctionDescriptor {
    pub name: String,
    pub min_argument_count: usize,
    pub max_argument_count: usize,
    pub parameters: Vec<ScriptHostParameterDescriptor>,
    pub return_value_kind: ScriptHostValueKind,
    #[serde(default)]
    pub return_type: ScriptHostTypeRef,
    pub required_capabilities: Vec<String>,
    pub documentation: Option<String>,
}

impl ScriptHostFunctionDescriptor {
    pub fn new(
        name: impl Into<String>,
        min_argument_count: usize,
        max_argument_count: usize,
        return_value_kind: ScriptHostValueKind,
    ) -> Self {
        Self {
            name: name.into(),
            min_argument_count,
            max_argument_count,
            parameters: Vec::new(),
            return_value_kind,
            return_type: ScriptHostTypeRef::from_value_kind(return_value_kind),
            required_capabilities: Vec::new(),
            documentation: None,
        }
    }

    pub fn with_return_type(mut self, return_type: ScriptHostTypeRef) -> Self {
        self.return_value_kind = return_type.value_kind;
        self.return_type = return_type;
        self
    }

    pub fn with_parameter(mut self, parameter: ScriptHostParameterDescriptor) -> Self {
        self.parameters.push(parameter);
        self
    }

    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.push(capability.into());
        self.required_capabilities.sort();
        self.required_capabilities.dedup();
        self
    }

    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptHostFieldDescriptor {
    pub name: String,
    pub value_kind: ScriptHostValueKind,
    #[serde(default)]
    pub type_ref: ScriptHostTypeRef,
    pub documentation: Option<String>,
}

impl ScriptHostFieldDescriptor {
    pub fn new(name: impl Into<String>, value_kind: ScriptHostValueKind) -> Self {
        Self {
            name: name.into(),
            value_kind,
            type_ref: ScriptHostTypeRef::from_value_kind(value_kind),
            documentation: None,
        }
    }

    pub fn with_type_ref(mut self, type_ref: ScriptHostTypeRef) -> Self {
        self.value_kind = type_ref.value_kind;
        self.type_ref = type_ref;
        self
    }

    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptHostTypeDescriptor {
    pub name: String,
    pub value_kind: ScriptHostValueKind,
    #[serde(default)]
    pub type_ref: ScriptHostTypeRef,
    #[serde(default)]
    pub prototype_kind: ScriptHostPrototypeKind,
    #[serde(default)]
    pub allow_value_construction: bool,
    pub fields: Vec<ScriptHostFieldDescriptor>,
    pub documentation: Option<String>,
}

impl ScriptHostTypeDescriptor {
    pub fn new(name: impl Into<String>, value_kind: ScriptHostValueKind) -> Self {
        Self {
            name: name.into(),
            value_kind,
            type_ref: ScriptHostTypeRef::from_value_kind(value_kind),
            prototype_kind: ScriptHostPrototypeKind::Struct,
            allow_value_construction: false,
            fields: Vec::new(),
            documentation: None,
        }
    }

    pub fn with_type_ref(mut self, type_ref: ScriptHostTypeRef) -> Self {
        self.value_kind = type_ref.value_kind;
        self.type_ref = type_ref;
        self
    }

    pub fn with_prototype_kind(mut self, prototype_kind: ScriptHostPrototypeKind) -> Self {
        self.prototype_kind = prototype_kind;
        self
    }

    pub fn allow_value_construction(mut self, allow_value_construction: bool) -> Self {
        self.allow_value_construction = allow_value_construction;
        self
    }

    pub fn with_field(mut self, field: ScriptHostFieldDescriptor) -> Self {
        self.fields.push(field);
        self
    }

    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptHostModuleDescriptor {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub functions: Vec<ScriptHostFunctionDescriptor>,
    pub types: Vec<ScriptHostTypeDescriptor>,
    pub documentation: Option<String>,
}

impl ScriptHostModuleDescriptor {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            capabilities: Vec::new(),
            functions: Vec::new(),
            types: Vec::new(),
            documentation: None,
        }
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self.capabilities.sort();
        self.capabilities.dedup();
        self
    }

    pub fn with_function(mut self, function: ScriptHostFunctionDescriptor) -> Self {
        self.functions.push(function);
        self
    }

    pub fn with_type(mut self, type_descriptor: ScriptHostTypeDescriptor) -> Self {
        self.types.push(type_descriptor);
        self
    }

    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScriptHostCallContext {
    pub module_name: String,
    pub function_name: String,
    pub arguments: Vec<ScriptHostValue>,
    pub granted_capabilities: Vec<String>,
}

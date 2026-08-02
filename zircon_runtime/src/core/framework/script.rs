//! Script-facing framework contracts shared by VM backends and host exports.

use std::any::Any;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

mod behavior_bridge;

pub use behavior_bridge::{
    SCRIPT_BEHAVIOR_BRIDGE_INTERFACE_ID, ScriptBehaviorBridge, ScriptBehaviorCallbackRef,
};

#[doc(hidden)]
pub mod __reflect {
    pub use zircon_runtime_interface::reflect::{
        ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectScriptVisibility,
        ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypeKind, ReflectTypePath,
        ReflectTypeRegistration,
    };
}

use __reflect::{ReflectError, ReflectScriptVisibility, ReflectTypeRegistration};

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

impl ScriptHostFromValue for Vec<u8> {
    fn script_host_type_ref() -> ScriptHostTypeRef {
        ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Bytes)
    }

    fn from_script_host_value(
        value: &ScriptHostValue,
        argument_index: usize,
    ) -> Result<Self, ScriptHostError> {
        match value {
            ScriptHostValue::Bytes(value) => Ok(value.clone()),
            value => Err(argument_type_error(
                argument_index,
                ScriptHostValueKind::Bytes,
                value,
            )),
        }
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
    fn reflect_type_registration() -> Result<ReflectTypeRegistration, ReflectError>;

    fn script_host_type_projection() -> ScriptHostTypeProjection;

    fn script_host_type_descriptor() -> Result<ScriptHostTypeDescriptor, ReflectError> {
        let registration = Self::reflect_type_registration()?;
        ScriptHostTypeDescriptor::from_reflect_registration(
            &registration,
            &Self::script_host_type_projection(),
        )
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptHostFieldProjection {
    pub name: String,
    pub value_kind: ScriptHostValueKind,
}

impl ScriptHostFieldProjection {
    pub fn new(name: impl Into<String>, value_kind: ScriptHostValueKind) -> Self {
        Self {
            name: name.into(),
            value_kind,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptHostTypeProjection {
    pub value_kind: ScriptHostValueKind,
    pub prototype_kind: ScriptHostPrototypeKind,
    pub allow_value_construction: bool,
    pub fields: Vec<ScriptHostFieldProjection>,
}

impl ScriptHostTypeProjection {
    pub fn new(value_kind: ScriptHostValueKind) -> Self {
        Self {
            value_kind,
            prototype_kind: ScriptHostPrototypeKind::Struct,
            allow_value_construction: false,
            fields: Vec::new(),
        }
    }

    pub fn with_prototype_kind(mut self, prototype_kind: ScriptHostPrototypeKind) -> Self {
        self.prototype_kind = prototype_kind;
        self
    }

    pub fn allow_value_construction(mut self, allow_value_construction: bool) -> Self {
        self.allow_value_construction = allow_value_construction;
        self
    }

    pub fn with_field(mut self, field: ScriptHostFieldProjection) -> Self {
        self.fields.push(field);
        self
    }
}

impl ScriptHostTypeDescriptor {
    pub fn new(name: impl Into<String>, value_kind: ScriptHostValueKind) -> Self {
        let name = name.into();
        Self {
            type_ref: ScriptHostTypeRef::new(value_kind, name.clone()),
            name,
            value_kind,
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

    pub fn from_reflect_registration(
        registration: &ReflectTypeRegistration,
        projection: &ScriptHostTypeProjection,
    ) -> Result<Self, ReflectError> {
        if registration.script_visibility != ReflectScriptVisibility::Public {
            return Err(ReflectError::InvalidRegistration {
                type_path: registration.type_path.type_path.clone(),
                reason: "script host projection requires public script visibility".to_string(),
            });
        }

        let mut projection_fields = HashMap::with_capacity(projection.fields.len());
        for projected_field in &projection.fields {
            if projection_fields
                .insert(projected_field.name.as_str(), projected_field.value_kind)
                .is_some()
            {
                return Err(ReflectError::InvalidRegistration {
                    type_path: registration.type_path.type_path.clone(),
                    reason: format!(
                        "script field projection `{}` is duplicated",
                        projected_field.name
                    ),
                });
            }
        }

        let mut descriptor = Self::new(&registration.display_name, projection.value_kind)
            .with_type_ref(ScriptHostTypeRef::new(
                projection.value_kind,
                &registration.display_name,
            ))
            .with_prototype_kind(projection.prototype_kind)
            .allow_value_construction(projection.allow_value_construction);
        descriptor
            .fields
            .reserve(registration.type_info.fields.len());
        let mut first_missing_reflected_field = None;
        for field in &registration.type_info.fields {
            let Some(value_kind) = projection_fields.remove(field.name.as_str()) else {
                first_missing_reflected_field.get_or_insert(field.name.as_str());
                continue;
            };
            let mut field_descriptor = ScriptHostFieldDescriptor::new(&field.name, value_kind)
                .with_type_ref(ScriptHostTypeRef::new(value_kind, &field.value_type_path));
            if let Some(documentation) = &field.documentation {
                field_descriptor = field_descriptor.with_documentation(documentation);
            }
            descriptor.fields.push(field_descriptor);
        }

        // Keep unknown projected fields ahead of missing ABI kinds in the error contract.
        if let Some(projected_field) = projection
            .fields
            .iter()
            .find(|field| projection_fields.contains_key(field.name.as_str()))
        {
            return Err(ReflectError::InvalidRegistration {
                type_path: registration.type_path.type_path.clone(),
                reason: format!(
                    "script field projection `{}` has no reflected field",
                    projected_field.name
                ),
            });
        }
        if let Some(field_name) = first_missing_reflected_field {
            return Err(ReflectError::InvalidRegistration {
                type_path: registration.type_path.type_path.clone(),
                reason: format!(
                    "reflected field `{field_name}` has no script ABI value-kind projection"
                ),
            });
        }
        if let Some(documentation) = &registration.documentation {
            descriptor.documentation = Some(documentation.clone());
        }
        Ok(descriptor)
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

pub struct ScriptHostCallFrame<'a> {
    pub module_name: &'a str,
    pub function_name: &'a str,
    pub arguments: &'a [ScriptHostValue],
    pub granted_capabilities: &'a [String],
    /// Runtime-owned data borrowed only for this synchronous host export call.
    runtime_context: Option<&'a dyn Any>,
}

impl<'a> ScriptHostCallFrame<'a> {
    pub(crate) fn new(
        module_name: &'a str,
        function_name: &'a str,
        arguments: &'a [ScriptHostValue],
        granted_capabilities: &'a [String],
        runtime_context: Option<&'a dyn Any>,
    ) -> Self {
        Self {
            module_name,
            function_name,
            arguments,
            granted_capabilities,
            runtime_context,
        }
    }

    pub(crate) fn runtime_context<T: Any>(&self) -> Option<&T> {
        self.runtime_context
            .and_then(|context| context.downcast_ref())
    }
}

impl std::fmt::Debug for ScriptHostCallFrame<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptHostCallFrame")
            .field("module_name", &self.module_name)
            .field("function_name", &self.function_name)
            .field("arguments", &self.arguments)
            .field("granted_capabilities", &self.granted_capabilities)
            .field(
                "runtime_context",
                &self.runtime_context.as_ref().map(|_| "<borrowed>"),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ScriptHostFromValue, ScriptHostIntoValue, ScriptHostTypeRef, ScriptHostValue,
        ScriptHostValueKind,
    };

    #[test]
    fn bytes_default_to_the_zr_vm_byte_array_type() {
        assert_eq!(
            ScriptHostValueKind::Bytes.default_zr_type_name(),
            "container.Array<uint>"
        );
        assert_eq!(
            ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Bytes).type_name,
            "container.Array<uint>"
        );
    }

    #[test]
    fn byte_vectors_round_trip_through_the_host_value_contract() {
        let bytes = vec![0, 104, 128, 255];
        let host_value = bytes.clone().into_script_host_value();

        assert_eq!(host_value, ScriptHostValue::Bytes(bytes.clone()));
        assert_eq!(
            Vec::<u8>::from_script_host_value(&host_value, 2).unwrap(),
            bytes
        );
        let error =
            Vec::<u8>::from_script_host_value(&ScriptHostValue::String("not-bytes".to_string()), 3)
                .unwrap_err();
        assert!(error.message.contains("argument 3"));
        assert!(error.message.contains("expected Bytes"));
    }
}

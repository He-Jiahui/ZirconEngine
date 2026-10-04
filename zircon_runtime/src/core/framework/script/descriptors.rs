use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::__reflect::{ReflectError, ReflectScriptVisibility, ReflectTypeRegistration};
use super::{ScriptHostPrototypeKind, ScriptHostTypeRef, ScriptHostValueKind};

/// 把反射注册信息与宏生成的脚本投影组合成宿主类型描述；此契约提供元数据，不注册运行期回调。
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

// capabilities 保持字典序且不重复；已有顺序时用二分定位，否则回退到排序去重。
fn insert_sorted_unique(values: &mut Vec<String>, value: String) {
    if values.windows(2).all(|pair| pair[0] < pair[1]) {
        if let Err(index) = values.binary_search(&value) {
            values.insert(index, value);
        }
        return;
    }

    values.push(value);
    values.sort();
    values.dedup();
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
        insert_sorted_unique(&mut self.required_capabilities, capability.into());
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

    /// 校验反射字段与脚本 ABI 投影一一对应，并沿用反射字段顺序和文档生成宿主描述。
    /// 调用方须提供公开给脚本的反射注册；缺字段、额外字段或重复投影都会返回注册错误。
    pub fn from_reflect_registration(
        registration: &ReflectTypeRegistration,
        projection: &ScriptHostTypeProjection,
    ) -> Result<Self, ReflectError> {
        if registration.script_visibility != ReflectScriptVisibility::Public {
            return Err(ReflectError::InvalidRegistration {
                type_path: registration.type_path.type_path().to_string(),
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
                    type_path: registration.type_path.type_path().to_string(),
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
                type_path: registration.type_path.type_path().to_string(),
                reason: format!(
                    "script field projection `{}` has no reflected field",
                    projected_field.name
                ),
            });
        }
        if let Some(field_name) = first_missing_reflected_field {
            return Err(ReflectError::InvalidRegistration {
                type_path: registration.type_path.type_path().to_string(),
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
        insert_sorted_unique(&mut self.capabilities, capability.into());
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

#[cfg(test)]
#[path = "tests/descriptors_optimization_tests.rs"]
mod optimization_tests;

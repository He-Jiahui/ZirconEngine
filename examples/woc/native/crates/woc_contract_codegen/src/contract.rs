use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractManifest {
    pub schema_version: u16,
    pub protocol_version: u16,
    pub namespace: String,
    pub finite_number_policy: String,
    pub limits: BTreeMap<String, u64>,
    pub reserved_ids: ReservedIds,
    pub enums: Vec<EnumDefinition>,
    pub contracts: Vec<ContractDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReservedIds {
    pub contracts: Vec<u16>,
    pub fields: Vec<u16>,
    pub enum_values: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumDefinition {
    pub name: String,
    pub id: u16,
    pub values: Vec<EnumValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumValue {
    pub name: String,
    pub id: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractDefinition {
    pub name: String,
    pub id: u16,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub id: u16,
    pub wire_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u64>,
}

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("failed to read contract manifest {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse contract manifest: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("unsupported {field} value {actual}, expected {expected}")]
    UnsupportedVersion {
        field: &'static str,
        actual: u16,
        expected: u16,
    },
    #[error("finite number policy must be reject, got {0}")]
    FiniteNumberPolicy(String),
    #[error("{kind} id {id} for {name} is reserved")]
    ReservedId {
        kind: &'static str,
        id: u16,
        name: String,
    },
    #[error("duplicate {kind} {value}")]
    Duplicate { kind: &'static str, value: String },
    #[error("{kind} {name} uses id zero")]
    ZeroId { kind: &'static str, name: String },
    #[error("field {contract}.{field} with type {wire_type} requires a positive max_length")]
    MissingBound {
        contract: String,
        field: String,
        wire_type: String,
    },
    #[error("vector field {contract}.{field} requires element_type")]
    MissingElementType { contract: String, field: String },
    #[error("field {contract}.{field} references unknown type {wire_type}")]
    UnknownType {
        contract: String,
        field: String,
        wire_type: String,
    },
    #[error("limit {name} must be positive")]
    InvalidLimit { name: String },
    #[error("manifest namespace must not be empty")]
    EmptyNamespace,
}

impl ContractManifest {
    pub fn from_slice(bytes: &[u8]) -> Result<Self, ContractError> {
        let manifest: Self = serde_json::from_slice(bytes)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        validate_schema_version(self.schema_version)?;
        validate_protocol_version(self.protocol_version)?;
        if self.namespace.trim().is_empty() {
            return Err(ContractError::EmptyNamespace);
        }
        if self.finite_number_policy != "reject" {
            return Err(ContractError::FiniteNumberPolicy(
                self.finite_number_policy.clone(),
            ));
        }
        for (name, value) in &self.limits {
            if *value == 0 {
                return Err(ContractError::InvalidLimit { name: name.clone() });
            }
        }

        let reserved_contracts = self
            .reserved_ids
            .contracts
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let reserved_fields = self
            .reserved_ids
            .fields
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let reserved_enum_values = self
            .reserved_ids
            .enum_values
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut type_names = primitive_types();
        for definition in &self.enums {
            type_names.insert(definition.name.as_str());
        }
        for definition in &self.contracts {
            type_names.insert(definition.name.as_str());
        }

        let mut contract_ids = BTreeSet::new();
        let mut contract_names = BTreeSet::new();
        for contract in &self.contracts {
            validate_id("contract", contract.id, &contract.name, &reserved_contracts)?;
            insert_unique("contract id", contract.id, &mut contract_ids)?;
            insert_unique("contract name", contract.name.clone(), &mut contract_names)?;
            let mut field_ids = BTreeSet::new();
            let mut field_names = BTreeSet::new();
            for field in &contract.fields {
                validate_id("field", field.id, &field.name, &reserved_fields)?;
                insert_unique("field id", field.id, &mut field_ids)?;
                insert_unique("field name", field.name.clone(), &mut field_names)?;
                if !type_names.contains(field.wire_type.as_str()) && field.wire_type != "vector" {
                    return Err(ContractError::UnknownType {
                        contract: contract.name.clone(),
                        field: field.name.clone(),
                        wire_type: field.wire_type.clone(),
                    });
                }
                if matches!(field.wire_type.as_str(), "bytes" | "string" | "vector")
                    && field.max_length.is_none_or(|bound| bound == 0)
                {
                    return Err(ContractError::MissingBound {
                        contract: contract.name.clone(),
                        field: field.name.clone(),
                        wire_type: field.wire_type.clone(),
                    });
                }
                if field.wire_type == "vector" {
                    let element_type = field.element_type.as_deref().ok_or_else(|| {
                        ContractError::MissingElementType {
                            contract: contract.name.clone(),
                            field: field.name.clone(),
                        }
                    })?;
                    if !type_names.contains(element_type) {
                        return Err(ContractError::UnknownType {
                            contract: contract.name.clone(),
                            field: field.name.clone(),
                            wire_type: element_type.to_string(),
                        });
                    }
                }
            }
        }

        let mut enum_ids = BTreeSet::new();
        let mut enum_names = BTreeSet::new();
        for definition in &self.enums {
            validate_id("enum", definition.id, &definition.name, &BTreeSet::new())?;
            insert_unique("enum id", definition.id, &mut enum_ids)?;
            insert_unique("enum name", definition.name.clone(), &mut enum_names)?;
            let mut value_ids = BTreeSet::new();
            let mut value_names = BTreeSet::new();
            for value in &definition.values {
                validate_id("enum value", value.id, &value.name, &reserved_enum_values)?;
                insert_unique("enum value id", value.id, &mut value_ids)?;
                insert_unique("enum value name", value.name.clone(), &mut value_names)?;
            }
        }
        Ok(())
    }

    pub fn fingerprint(&self) -> Result<[u8; 32], ContractError> {
        let value = serde_json::to_value(self)?;
        let canonical = serde_json::to_vec(&value)?;
        Ok(Sha256::digest(canonical).into())
    }
}

pub fn load_contract_manifest(path: impl AsRef<Path>) -> Result<ContractManifest, ContractError> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|source| ContractError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    ContractManifest::from_slice(&bytes)
}

fn primitive_types() -> BTreeSet<&'static str> {
    [
        "bool", "u8", "u16", "u32", "u64", "i32", "i64", "f64", "bytes", "string",
    ]
    .into_iter()
    .collect()
}

fn validate_schema_version(actual: u16) -> Result<(), ContractError> {
    if actual != 1 {
        return Err(ContractError::UnsupportedVersion {
            field: "schema_version",
            actual,
            expected: 1,
        });
    }
    Ok(())
}

fn validate_protocol_version(actual: u16) -> Result<(), ContractError> {
    if actual == 0 {
        return Err(ContractError::UnsupportedVersion {
            field: "protocol_version",
            actual,
            expected: 1,
        });
    }
    Ok(())
}

fn validate_id(
    kind: &'static str,
    id: u16,
    name: &str,
    reserved: &BTreeSet<u16>,
) -> Result<(), ContractError> {
    if id == 0 {
        return Err(ContractError::ZeroId {
            kind,
            name: name.to_string(),
        });
    }
    if reserved.contains(&id) {
        return Err(ContractError::ReservedId {
            kind,
            id,
            name: name.to_string(),
        });
    }
    Ok(())
}

fn insert_unique<T: Ord + Clone + ToString>(
    kind: &'static str,
    value: T,
    set: &mut BTreeSet<T>,
) -> Result<(), ContractError> {
    if !set.insert(value.clone()) {
        return Err(ContractError::Duplicate {
            kind,
            value: value.to_string(),
        });
    }
    Ok(())
}

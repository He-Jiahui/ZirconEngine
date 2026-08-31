use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;
use zircon_runtime_interface::ui::binding::{
    UiModelContextLayer, UiModelContextPatch, UiModelFieldId, UiModelFieldSchema,
    UiModelProviderKey, UiModelProviderSchema, UiModelSchema, UiModelSchemaKey,
    UiResolvedModelContext,
};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiModelSchemaRegistrationError {
    #[error("model schema {schema:?} must declare at least one field")]
    EmptySchema { schema: UiModelSchemaKey },
    #[error("model schema {schema:?} declares duplicate field {field}")]
    DuplicateField {
        schema: UiModelSchemaKey,
        field: UiModelFieldId,
    },
    #[error("model schema identity {schema:?} is already registered with different fields")]
    SchemaIdentityCollision { schema: UiModelSchemaKey },
    #[error("model provider identity {provider:?} is already registered with a different schema")]
    ProviderIdentityCollision { provider: UiModelProviderKey },
    #[error("model provider {provider:?} references missing schema {schema:?}")]
    MissingModelSchema {
        provider: UiModelProviderKey,
        schema: UiModelSchemaKey,
    },
    #[error("model provider {provider:?} is not registered")]
    UnknownProvider { provider: UiModelProviderKey },
    #[error("model provider {provider:?} schema has no field {field}")]
    UnknownField {
        provider: UiModelProviderKey,
        field: UiModelFieldId,
    },
    #[error("model context layer {layer:?} references unregistered provider {provider:?}")]
    UnknownContextProvider {
        layer: UiModelContextLayer,
        provider: UiModelProviderKey,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RegisteredModelSchema {
    schema: UiModelSchema,
    field_indices: BTreeMap<UiModelFieldId, usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiModelSchemaRegistry {
    schemas: BTreeMap<UiModelSchemaKey, RegisteredModelSchema>,
    providers: BTreeMap<UiModelProviderKey, UiModelProviderSchema>,
    revision: u64,
}

impl UiModelSchemaRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_schema(
        &mut self,
        schema: UiModelSchema,
    ) -> Result<bool, UiModelSchemaRegistrationError> {
        if schema.fields.is_empty() {
            return Err(UiModelSchemaRegistrationError::EmptySchema {
                schema: schema.key().clone(),
            });
        }

        let mut unique_fields = BTreeSet::new();
        let mut field_indices = BTreeMap::new();
        for (index, field) in schema.fields.iter().enumerate() {
            if !unique_fields.insert(field.id.clone()) {
                return Err(UiModelSchemaRegistrationError::DuplicateField {
                    schema: schema.key().clone(),
                    field: field.id.clone(),
                });
            }
            field_indices.insert(field.id.clone(), index);
        }

        if let Some(registered) = self.schemas.get(schema.key()) {
            if registered.schema == schema {
                return Ok(false);
            }
            return Err(UiModelSchemaRegistrationError::SchemaIdentityCollision {
                schema: schema.key().clone(),
            });
        }

        self.schemas.insert(
            schema.key().clone(),
            RegisteredModelSchema {
                schema,
                field_indices,
            },
        );
        self.revision = self.revision.saturating_add(1);
        Ok(true)
    }

    pub fn register_provider(
        &mut self,
        provider: UiModelProviderSchema,
    ) -> Result<bool, UiModelSchemaRegistrationError> {
        if !self.schemas.contains_key(&provider.model_schema) {
            return Err(UiModelSchemaRegistrationError::MissingModelSchema {
                provider: provider.key().clone(),
                schema: provider.model_schema.clone(),
            });
        }
        if let Some(registered) = self.providers.get(provider.key()) {
            if registered == &provider {
                return Ok(false);
            }
            return Err(UiModelSchemaRegistrationError::ProviderIdentityCollision {
                provider: provider.key().clone(),
            });
        }

        self.providers.insert(provider.key().clone(), provider);
        self.revision = self.revision.saturating_add(1);
        Ok(true)
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub fn schema(&self, key: &UiModelSchemaKey) -> Option<&UiModelSchema> {
        self.schemas.get(key).map(|registered| &registered.schema)
    }

    pub fn provider(&self, key: &UiModelProviderKey) -> Option<&UiModelProviderSchema> {
        self.providers.get(key)
    }

    pub fn schema_keys(&self) -> impl ExactSizeIterator<Item = &UiModelSchemaKey> {
        self.schemas.keys()
    }

    pub fn provider_keys(&self) -> impl ExactSizeIterator<Item = &UiModelProviderKey> {
        self.providers.keys()
    }

    pub fn resolve_field(
        &self,
        provider_key: &UiModelProviderKey,
        field_id: &UiModelFieldId,
    ) -> Result<&UiModelFieldSchema, UiModelSchemaRegistrationError> {
        let provider = self.providers.get(provider_key).ok_or_else(|| {
            UiModelSchemaRegistrationError::UnknownProvider {
                provider: provider_key.clone(),
            }
        })?;
        let registered = self.schemas.get(&provider.model_schema).ok_or_else(|| {
            UiModelSchemaRegistrationError::MissingModelSchema {
                provider: provider_key.clone(),
                schema: provider.model_schema.clone(),
            }
        })?;
        let field_index = registered.field_indices.get(field_id).ok_or_else(|| {
            UiModelSchemaRegistrationError::UnknownField {
                provider: provider_key.clone(),
                field: field_id.clone(),
            }
        })?;
        Ok(&registered.schema.fields[*field_index])
    }

    pub fn resolve_model_context(
        &self,
        parent: Option<&UiResolvedModelContext>,
        patch: &UiModelContextPatch,
    ) -> Result<UiResolvedModelContext, UiModelSchemaRegistrationError> {
        let resolved = UiResolvedModelContext::resolve(parent, patch);
        for (layer, provider) in resolved.providers() {
            if !self.providers.contains_key(provider) {
                return Err(UiModelSchemaRegistrationError::UnknownContextProvider {
                    layer,
                    provider: provider.clone(),
                });
            }
        }
        Ok(resolved)
    }
}

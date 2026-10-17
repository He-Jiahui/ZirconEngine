//! 保存模型模式、字段索引和提供者的精确版本关系，并校验解析后的上下文。
//! 目录拥有静态描述符；字段访问能力属于描述数据，实际模型读写由调用方实现。

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;
use zircon_runtime_interface::ui::binding::{
    UiModelContextLayer, UiModelContextPatch, UiModelFieldId, UiModelFieldSchema,
    UiModelProviderKey, UiModelProviderSchema, UiModelSchema, UiModelSchemaKey,
    UiResolvedModelContext,
};

/// 模式或提供者注册、字段解析与上下文校验失败；错误保留拥有型键和必要的层、字段信息。
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

/// 拥有原模式及其字段索引；索引只在字段唯一性校验通过后与模式一起存入。
#[derive(Clone, Debug, PartialEq, Eq)]
struct RegisteredModelSchema {
    schema: UiModelSchema,
    // 索引位置对应同一 schema.fields；注册后不改写其内容，保证 resolve_field 可直接取下标。
    field_indices: BTreeMap<UiModelFieldId, usize>,
}

/// 绑定编译/上下文解析使用的静态 schema 目录；描述字段访问能力，不持有实际模型值。
/// provider 绑定精确 schema 版本；同一版本的结构不可改写，版本可并存以支持既有绑定。
///
/// # 所有权与借用
/// 注册接收拥有型描述符，查询和键迭代器借用本表；上下文解析则返回独立拥有的键集合。
/// 修订号只记录本目录的新增项并使用饱和加法，不代表实际模型值的版本。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiModelSchemaRegistry {
    schemas: BTreeMap<UiModelSchemaKey, RegisteredModelSchema>,
    providers: BTreeMap<UiModelProviderKey, UiModelProviderSchema>,
    revision: u64,
}

impl UiModelSchemaRegistry {
    /// 创建没有模式、提供者且修订号为零的目录。
    pub fn new() -> Self {
        Self::default()
    }

    /// 在注册提供者前录入一个精确版本的模式，并建立字段到原数组位置的索引。
    ///
    /// # 参数与返回
    /// `schema` 必须至少有一个字段且字段身份唯一；成功新增返回 `true`，完全相同内容
    /// 重复录入返回 `false`。空字段、重复字段或同键异内容返回错误，均不修改目录。
    /// 描述符在新增时移入目录；只有新增才推进修订号。
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

    /// 先注册目标 schema 再注册 provider；重复身份只能复用相同内容，避免上下文解析随注册顺序漂移。
    ///
    /// # 参数与返回
    /// `provider` 的目标模式键须已存在；新增时接管描述符并返回 `true`，内容相同的
    /// 重复身份返回 `false`。缺少模式或同身份异内容返回错误，目录及修订号保持原状。
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

    /// 返回新增模式或提供者时饱和递增的目录修订号；幂等注册和失败均不推进。
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// 按精确版本键查询模式；不存在时为 None，成功引用借用自本目录。
    pub fn schema(&self, key: &UiModelSchemaKey) -> Option<&UiModelSchema> {
        self.schemas.get(key).map(|registered| &registered.schema)
    }

    /// 按精确版本键查询提供者描述符；不存在时为 None，成功引用借用自本目录。
    pub fn provider(&self, key: &UiModelProviderKey) -> Option<&UiModelProviderSchema> {
        self.providers.get(key)
    }

    /// 借用本表并按键的 Ord 顺序迭代模式身份；迭代器不复制键，也不按注册顺序输出。
    pub fn schema_keys(&self) -> impl ExactSizeIterator<Item = &UiModelSchemaKey> {
        self.schemas.keys()
    }

    /// 借用本表并按键的 Ord 顺序迭代提供者身份；存续期间受本表借用约束。
    pub fn provider_keys(&self) -> impl ExactSizeIterator<Item = &UiModelProviderKey> {
        self.providers.keys()
    }

    /// 通过提供者绑定的精确模式版本解析字段描述符。
    ///
    /// # 参数与返回
    /// `provider_key` 和 `field_id` 仅在查询期间借用；依次校验提供者、目标模式和字段索引，
    /// 首个缺失项返回相应错误。成功引用借用自目录，不读取模型值，也不执行字段权限。
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

    /// 合并继承、覆盖与清除后验证全部有效 provider，包括继承项；不能把父上下文当作已可信的缓存。
    ///
    /// # 参数与返回
    /// `parent` 为可选的继承输入，`patch` 按层选择继承、覆盖或清除；两者均只在调用中借用。
    /// 返回独立拥有的有效提供者键集合；任一有效层引用未知的精确版本键即返回错误。
    /// 本方法不修改父输入、补丁或目录，返回值也不保留它们的借用。
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

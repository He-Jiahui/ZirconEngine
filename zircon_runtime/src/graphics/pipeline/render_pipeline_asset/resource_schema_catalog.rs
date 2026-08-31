use crate::core::framework::render::RenderFrameExtract;
use crate::graphics::RenderResourceSchema;
use crate::graphics::pipeline::RenderPipelineCompileOptions;
use crate::rhi::{BufferDesc, TextureDesc};

use super::resource_descriptors::{
    buffer_desc_from_schema, builtin_buffer_desc_for, builtin_external_texture_desc_for,
    builtin_texture_desc_for, texture_desc_from_schema,
};

/// Resolves the physical contract for resources authored by a render pipeline.
///
/// Built-in product resources use this one catalog. Every other transient
/// resource must arrive with a schema owned by its feature or plugin; a label
/// is never a descriptor inference input.
pub(super) struct RenderResourceSchemaCatalog<'a> {
    extract: &'a RenderFrameExtract,
    options: &'a RenderPipelineCompileOptions,
}

impl<'a> RenderResourceSchemaCatalog<'a> {
    pub(super) const fn new(
        extract: &'a RenderFrameExtract,
        options: &'a RenderPipelineCompileOptions,
    ) -> Self {
        Self { extract, options }
    }

    pub(super) fn texture_desc(
        &self,
        name: &str,
        schema: Option<RenderResourceSchema>,
    ) -> Result<TextureDesc, String> {
        match schema {
            Some(schema) => texture_desc_from_schema(name, schema, self.extract),
            None => builtin_texture_desc_for(name, self.extract, self.options)
                .ok_or_else(|| missing_schema_error("transient texture", name)),
        }
    }

    pub(super) fn buffer_desc(
        &self,
        name: &str,
        schema: Option<RenderResourceSchema>,
        minimum_size_bytes: Option<u64>,
    ) -> Result<BufferDesc, String> {
        match schema {
            Some(schema) => buffer_desc_from_schema(name, schema, minimum_size_bytes),
            None => builtin_buffer_desc_for(name, self.extract, minimum_size_bytes)?
                .ok_or_else(|| missing_schema_error("transient buffer", name)),
        }
    }

    pub(super) fn external_texture_desc(
        &self,
        name: &str,
        schema: Option<RenderResourceSchema>,
    ) -> Result<Option<TextureDesc>, String> {
        match schema {
            Some(schema) => texture_desc_from_schema(name, schema, self.extract).map(Some),
            None => Ok(builtin_external_texture_desc_for(
                name,
                self.extract,
                self.options,
            )),
        }
    }
}

fn missing_schema_error(resource_kind: &str, name: &str) -> String {
    format!(
        "{resource_kind} resource `{name}` requires an explicit RenderResourceSchema; only catalog-defined builtin resources may omit one"
    )
}

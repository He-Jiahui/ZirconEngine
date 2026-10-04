use crate::graphics::shader::template::{ShaderTemplateReflection, ShaderVertexInputScalarKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MeshShaderVertexAttribute {
    location: u32,
    scalar_kind: ShaderVertexInputScalarKind,
}

impl MeshShaderVertexAttribute {
    pub(super) const fn new(location: u32, scalar_kind: ShaderVertexInputScalarKind) -> Self {
        Self {
            location,
            scalar_kind,
        }
    }
}

/// 将真实 GPU 顶点布局投影为 shader @location 与标量类别约束。
/// 创建管线前必须验证；速度 pass 还包含前一帧位置槽。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MeshShaderVertexLayoutContract {
    attributes: Vec<MeshShaderVertexAttribute>,
}

impl MeshShaderVertexLayoutContract {
    pub(super) fn try_new(
        attributes: impl IntoIterator<Item = MeshShaderVertexAttribute>,
    ) -> Result<Self, String> {
        let mut attributes = attributes.into_iter().collect::<Vec<_>>();
        attributes.sort_unstable_by_key(|attribute| attribute.location);
        for pair in attributes.windows(2) {
            if pair[0].location == pair[1].location {
                return Err(format!(
                    "Mesh vertex layout contains duplicate @location({})",
                    pair[0].location
                ));
            }
        }
        Ok(Self { attributes })
    }

    pub(super) fn validate(
        &self,
        reflection: &ShaderTemplateReflection,
        vertex_entry: &str,
    ) -> Result<(), String> {
        reflection.validate_vertex_input_stage_interface(vertex_entry, |location| {
            self.scalar_kind_at(location)
        })
    }

    pub(super) fn scalar_kind_at(&self, location: u32) -> Option<ShaderVertexInputScalarKind> {
        self.attributes
            .binary_search_by_key(&location, |attribute| attribute.location)
            .ok()
            .map(|index| self.attributes[index].scalar_kind)
    }

    #[cfg(test)]
    pub(super) fn attribute_count(&self) -> usize {
        self.attributes.len()
    }
}

#[cfg(test)]
#[path = "tests/mesh_shader_vertex_contract.rs"]
mod tests;

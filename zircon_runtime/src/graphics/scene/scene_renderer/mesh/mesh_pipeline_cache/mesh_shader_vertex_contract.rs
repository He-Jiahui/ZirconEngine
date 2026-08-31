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
mod tests {
    use super::{
        MeshShaderVertexAttribute, MeshShaderVertexLayoutContract, ShaderVertexInputScalarKind,
    };

    #[test]
    fn vertex_contract_sorts_attributes_for_bounded_lookup() {
        let contract = MeshShaderVertexLayoutContract::try_new([
            MeshShaderVertexAttribute::new(8, ShaderVertexInputScalarKind::Float),
            MeshShaderVertexAttribute::new(0, ShaderVertexInputScalarKind::Float),
            MeshShaderVertexAttribute::new(3, ShaderVertexInputScalarKind::Uint),
        ])
        .expect("unique attributes");

        assert_eq!(
            contract.scalar_kind_at(3),
            Some(ShaderVertexInputScalarKind::Uint)
        );
        assert_eq!(contract.scalar_kind_at(7), None);
    }

    #[test]
    fn vertex_contract_rejects_duplicate_locations() {
        let error = MeshShaderVertexLayoutContract::try_new([
            MeshShaderVertexAttribute::new(0, ShaderVertexInputScalarKind::Float),
            MeshShaderVertexAttribute::new(0, ShaderVertexInputScalarKind::Uint),
        ])
        .expect_err("one shader location cannot have two vertex attributes");

        assert!(error.contains("@location(0)"), "unexpected error: {error}");
    }
}

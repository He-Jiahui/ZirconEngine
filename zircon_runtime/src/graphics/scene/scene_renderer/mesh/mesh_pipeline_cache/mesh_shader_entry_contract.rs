#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShaderEntryStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MeshShaderProgramKind {
    Base,
    GBuffer,
    DepthPrepass { alpha_masked: bool },
    ShadowDepth,
    ShadowDepthAlphaMask,
    Velocity,
    TaaReactiveMask,
    TaaReactiveMaterialMask,
    HitProxy,
    Oit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MeshShaderEntryContract {
    vertex_entry: &'static str,
    fragment_entry: Option<&'static str>,
}

impl MeshShaderEntryContract {
    pub(super) const fn for_program(kind: MeshShaderProgramKind) -> Self {
        match kind {
            MeshShaderProgramKind::DepthPrepass {
                alpha_masked: false,
            }
            | MeshShaderProgramKind::ShadowDepth => Self::vertex_only("vs_main"),
            MeshShaderProgramKind::TaaReactiveMask => {
                Self::vertex_fragment("vs_main", "fs_taa_reactive_mask")
            }
            MeshShaderProgramKind::TaaReactiveMaterialMask => {
                Self::vertex_fragment("vs_main", "fs_taa_reactive_material_mask")
            }
            MeshShaderProgramKind::Oit => Self::vertex_fragment("vs_main", "fs_oit"),
            MeshShaderProgramKind::HitProxy => Self::vertex_fragment("vs_main", "fs_main"),
            MeshShaderProgramKind::Base
            | MeshShaderProgramKind::GBuffer
            | MeshShaderProgramKind::DepthPrepass { alpha_masked: true }
            | MeshShaderProgramKind::ShadowDepthAlphaMask
            | MeshShaderProgramKind::Velocity => Self::vertex_fragment("vs_main", "fs_main"),
        }
    }

    pub(super) const fn vertex_only(vertex_entry: &'static str) -> Self {
        Self {
            vertex_entry,
            fragment_entry: None,
        }
    }

    pub(super) const fn vertex_fragment(
        vertex_entry: &'static str,
        fragment_entry: &'static str,
    ) -> Self {
        Self {
            vertex_entry,
            fragment_entry: Some(fragment_entry),
        }
    }

    pub(super) fn validate(
        self,
        mut contains_entry: impl FnMut(ShaderEntryStage, &str) -> bool,
    ) -> Result<(), String> {
        if !contains_entry(ShaderEntryStage::Vertex, self.vertex_entry) {
            return Err(format!(
                "shader is missing required @vertex entry `{}`",
                self.vertex_entry
            ));
        }
        if let Some(fragment_entry) = self.fragment_entry {
            if !contains_entry(ShaderEntryStage::Fragment, fragment_entry) {
                return Err(format!(
                    "shader is missing required @fragment entry `{fragment_entry}`"
                ));
            }
        }
        Ok(())
    }

    pub(super) fn try_for_each_required_entry(
        self,
        mut visit: impl FnMut(ShaderEntryStage, &str) -> Result<(), String>,
    ) -> Result<(), String> {
        visit(ShaderEntryStage::Vertex, self.vertex_entry)?;
        if let Some(fragment_entry) = self.fragment_entry {
            visit(ShaderEntryStage::Fragment, fragment_entry)?;
        }
        Ok(())
    }

    pub(super) const fn vertex_fragment_entries(self) -> Option<(&'static str, &'static str)> {
        match self.fragment_entry {
            Some(fragment_entry) => Some((self.vertex_entry, fragment_entry)),
            None => None,
        }
    }

    pub(super) const fn vertex_entry(self) -> &'static str {
        self.vertex_entry
    }
}

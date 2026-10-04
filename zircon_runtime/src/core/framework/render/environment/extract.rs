use super::{
    IblBakeArtifactContents, IblBakeArtifactRequest, IblBakeKey, LightProbeGridData,
    LightmapConsumeContract, LightmapContractValidationError, ReflectionProbeData, SkyboxSettings,
    SourceCubemapEnvironment, SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};

/// 场景提取阶段交给渲染帧的环境快照，合并天空、反射探针和同代烘焙照明。
/// 构建后由帧提交路径读取；源 cubemap 的上传字节应在进入渲染提交前准备好。
#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentExtract {
    pub skybox: SkyboxSettings,
    pub probes: Vec<ReflectionProbeData>,
    pub baked_lighting: Option<LightmapConsumeContract>,
    pub probe_grid: Option<LightProbeGridData>,
}

impl EnvironmentExtract {
    pub fn disabled() -> Self {
        Self {
            skybox: SkyboxSettings::none(),
            probes: Vec::new(),
            baked_lighting: None,
            probe_grid: None,
        }
    }

    pub fn procedural_default() -> Self {
        Self {
            skybox: SkyboxSettings::procedural_default(),
            probes: Vec::new(),
            baked_lighting: None,
            probe_grid: None,
        }
    }

    pub fn source_cubemap(source_cubemap: SourceCubemapEnvironment) -> Self {
        Self {
            skybox: SkyboxSettings::source_cubemap(source_cubemap),
            probes: Vec::new(),
            baked_lighting: None,
            probe_grid: None,
        }
    }

    pub fn from_preview_skybox_enabled(enabled: bool) -> Self {
        if enabled {
            Self::procedural_default()
        } else {
            Self::disabled()
        }
    }

    pub fn skybox_enabled(&self) -> bool {
        self.skybox.is_enabled()
    }

    pub fn with_reflection_probes(mut self, probes: Vec<ReflectionProbeData>) -> Self {
        self.probes = probes;
        self
    }

    pub fn reflection_probes(&self) -> &[ReflectionProbeData] {
        &self.probes
    }

    /// 仅接受同一光照代的 atlas 与探针网格，避免帧内混用不同烘焙结果。
    /// 调用方应在场景提取时处理错误，不能把未校验的组合直接交给渲染器。
    pub fn try_with_baked_lighting(
        mut self,
        baked_lighting: LightmapConsumeContract,
        probe_grid: Option<LightProbeGridData>,
    ) -> Result<Self, LightmapContractValidationError> {
        baked_lighting.validate()?;
        if let Some(grid) = &probe_grid {
            grid.validate()?;
            if grid.light_set_generation != baked_lighting.light_set_generation {
                return Err(LightmapContractValidationError::GenerationMismatch);
            }
        }
        self.baked_lighting = Some(baked_lighting);
        self.probe_grid = probe_grid;
        Ok(self)
    }

    pub fn baked_lighting(&self) -> Option<&LightmapConsumeContract> {
        self.baked_lighting.as_ref()
    }

    pub fn light_probe_grid(&self) -> Option<&LightProbeGridData> {
        self.probe_grid.as_ref()
    }

    pub fn ibl_bake_key(&self) -> Option<IblBakeKey> {
        self.skybox.ibl_bake_key()
    }

    pub fn source_cubemap_ibl_bake_request(
        &self,
        required_contents: IblBakeArtifactContents,
    ) -> Option<IblBakeArtifactRequest> {
        self.skybox
            .source_cubemap_environment()
            .map(|environment| environment.ibl_bake_artifact_request(required_contents))
    }
}

impl Default for EnvironmentExtract {
    fn default() -> Self {
        Self::disabled()
    }
}

#[cfg(test)]
#[path = "tests/extract.rs"]
mod tests;

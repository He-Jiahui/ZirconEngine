use super::ibl_bake_recipe::{
    IblBakeDiffuseIntegrator, IblBakeRecipe, IblBakeRecipeIdentity, CANONICAL_IBL_BAKE_RECIPE,
};

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnvironmentBrdfLutIntegrator {
    GgxJointSmithSplitSum = 1,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnvironmentBrdfLutFormat {
    Rg16Float = 1,
}

impl EnvironmentBrdfLutFormat {
    pub const fn texel_size_bytes(self) -> u32 {
        match self {
            Self::Rg16Float => 4,
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnvironmentPbrEnergyMode {
    SingleScatterSplitSum = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EnvironmentBrdfLutRecipeIdentity {
    algorithm_version: u64,
    extent: [u32; 2],
    sample_count: u32,
    integrator: EnvironmentBrdfLutIntegrator,
    output_format: EnvironmentBrdfLutFormat,
}

impl EnvironmentBrdfLutRecipeIdentity {
    pub const fn algorithm_version(self) -> u64 {
        self.algorithm_version
    }

    pub const fn extent(self) -> [u32; 2] {
        self.extent
    }

    pub const fn sample_count(self) -> u32 {
        self.sample_count
    }

    pub const fn integrator(self) -> EnvironmentBrdfLutIntegrator {
        self.integrator
    }

    pub const fn output_format(self) -> EnvironmentBrdfLutFormat {
        self.output_format
    }
}

/// Device-global split-sum LUT policy. This identity is deliberately independent
/// from producer-specific `.zribl` artifact identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EnvironmentBrdfLutRecipe {
    identity: EnvironmentBrdfLutRecipeIdentity,
}

impl EnvironmentBrdfLutRecipe {
    pub const fn identity(self) -> EnvironmentBrdfLutRecipeIdentity {
        self.identity
    }

    pub const fn algorithm_version(self) -> u64 {
        self.identity.algorithm_version()
    }

    pub const fn extent(self) -> [u32; 2] {
        self.identity.extent()
    }

    pub const fn width(self) -> u32 {
        self.extent()[0]
    }

    pub const fn height(self) -> u32 {
        self.extent()[1]
    }

    pub const fn sample_count(self) -> u32 {
        self.identity.sample_count()
    }

    pub const fn integrator(self) -> EnvironmentBrdfLutIntegrator {
        self.identity.integrator()
    }

    pub const fn output_format(self) -> EnvironmentBrdfLutFormat {
        self.identity.output_format()
    }

    pub const fn expected_byte_len(self) -> usize {
        self.width() as usize
            * self.height() as usize
            * self.output_format().texel_size_bytes() as usize
    }
}

/// 供端到端环境 PBR 产物比较的复合身份；CPU 导入与运行时 GPU 烘焙保留各自的积分器身份。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EnvironmentPbrRecipeIdentity {
    ibl_bake_recipe: IblBakeRecipeIdentity,
    brdf_lut_recipe: EnvironmentBrdfLutRecipeIdentity,
    base_lobe_energy_mode: EnvironmentPbrEnergyMode,
}

impl EnvironmentPbrRecipeIdentity {
    pub const fn ibl_bake_recipe(self) -> IblBakeRecipeIdentity {
        self.ibl_bake_recipe
    }

    pub const fn brdf_lut_recipe(self) -> EnvironmentBrdfLutRecipeIdentity {
        self.brdf_lut_recipe
    }

    pub const fn base_lobe_energy_mode(self) -> EnvironmentPbrEnergyMode {
        self.base_lobe_energy_mode
    }
}

/// Composite product contract. Sub-artifacts keep their own lifetimes and use
/// this type only when an end-to-end environment/PBR identity is required.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvironmentPbrRecipe {
    ibl_bake_recipe: IblBakeRecipe,
    brdf_lut_recipe: EnvironmentBrdfLutRecipe,
    base_lobe_energy_mode: EnvironmentPbrEnergyMode,
}

impl EnvironmentPbrRecipe {
    pub const fn ibl_bake_recipe(self) -> IblBakeRecipe {
        self.ibl_bake_recipe
    }

    pub const fn brdf_lut_recipe(self) -> EnvironmentBrdfLutRecipe {
        self.brdf_lut_recipe
    }

    pub const fn base_lobe_energy_mode(self) -> EnvironmentPbrEnergyMode {
        self.base_lobe_energy_mode
    }

    pub const fn asset_recipe_identity(self) -> EnvironmentPbrRecipeIdentity {
        self.identity(IblBakeDiffuseIntegrator::AssetImporterCpuSolidAngle)
    }

    pub const fn runtime_recipe_identity(self) -> EnvironmentPbrRecipeIdentity {
        self.identity(IblBakeDiffuseIntegrator::RendererGpuRuntimeHammersley)
    }

    pub const fn identity(
        self,
        diffuse_integrator: IblBakeDiffuseIntegrator,
    ) -> EnvironmentPbrRecipeIdentity {
        EnvironmentPbrRecipeIdentity {
            ibl_bake_recipe: self.ibl_bake_recipe.identity(diffuse_integrator),
            brdf_lut_recipe: self.brdf_lut_recipe.identity(),
            base_lobe_energy_mode: self.base_lobe_energy_mode,
        }
    }
}

pub const CANONICAL_ENVIRONMENT_BRDF_LUT_ALGORITHM_VERSION: u64 = 2026_08_31_0001;
pub const ENVIRONMENT_BRDF_LUT_WIDTH: u32 = 128;
pub const ENVIRONMENT_BRDF_LUT_HEIGHT: u32 = 32;
pub const ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT: u32 = 128;

pub const CANONICAL_ENVIRONMENT_BRDF_LUT_RECIPE: EnvironmentBrdfLutRecipe =
    EnvironmentBrdfLutRecipe {
        identity: EnvironmentBrdfLutRecipeIdentity {
            algorithm_version: CANONICAL_ENVIRONMENT_BRDF_LUT_ALGORITHM_VERSION,
            extent: [ENVIRONMENT_BRDF_LUT_WIDTH, ENVIRONMENT_BRDF_LUT_HEIGHT],
            sample_count: ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT,
            integrator: EnvironmentBrdfLutIntegrator::GgxJointSmithSplitSum,
            output_format: EnvironmentBrdfLutFormat::Rg16Float,
        },
    };

pub const CANONICAL_ENVIRONMENT_PBR_RECIPE: EnvironmentPbrRecipe = EnvironmentPbrRecipe {
    ibl_bake_recipe: CANONICAL_IBL_BAKE_RECIPE,
    brdf_lut_recipe: CANONICAL_ENVIRONMENT_BRDF_LUT_RECIPE,
    base_lobe_energy_mode: EnvironmentPbrEnergyMode::SingleScatterSplitSum,
};

#[cfg(test)]
#[path = "tests/environment_pbr_recipe.rs"]
mod tests;

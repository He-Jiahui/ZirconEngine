use crate::core::math::Real;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IblBakeOutputFormat {
    Rgba16Float = 1,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IblBakePmremIntegrator {
    GgxFilteredImportanceSampling = 1,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IblBakeDiffuseRepresentation {
    Sh9WithOptionalIrradianceCube = 1,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IblBakeDiffuseIntegrator {
    AssetImporterCpuSolidAngle = 1,
    RendererGpuRuntimeHammersley = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IblBakeRecipeIdentity {
    algorithm_version: u64,
    pmrem_integrator: IblBakePmremIntegrator,
    diffuse_integrator: IblBakeDiffuseIntegrator,
    output_format: IblBakeOutputFormat,
}

impl IblBakeRecipeIdentity {
    pub const fn new(
        algorithm_version: u64,
        pmrem_integrator: IblBakePmremIntegrator,
        diffuse_integrator: IblBakeDiffuseIntegrator,
        output_format: IblBakeOutputFormat,
    ) -> Self {
        Self {
            algorithm_version,
            pmrem_integrator,
            diffuse_integrator,
            output_format,
        }
    }

    pub const fn algorithm_version(self) -> u64 {
        self.algorithm_version
    }

    pub const fn pmrem_integrator(self) -> IblBakePmremIntegrator {
        self.pmrem_integrator
    }

    pub const fn diffuse_integrator(self) -> IblBakeDiffuseIntegrator {
        self.diffuse_integrator
    }

    pub const fn output_format(self) -> IblBakeOutputFormat {
        self.output_format
    }
}

/// Immutable artifact and sampling policy shared by every IBL implementation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IblBakeRecipe {
    algorithm_version: u64,
    diffuse_source_face_size: u32,
    irradiance_cube_face_size: u32,
    pmrem_fast_sample_count: u32,
    pmrem_normal_sample_count: u32,
    pmrem_rough_sample_count: u32,
    runtime_diffuse_sample_count: u32,
    pmrem_low_roughness_threshold: Real,
    pmrem_high_roughness_threshold: Real,
    full_roughness_cosine_threshold: Real,
    fis_solid_angle_texel_scale: Real,
    roughest_mip_offset: Real,
    roughness_mip_scale: Real,
    output_format: IblBakeOutputFormat,
    pmrem_integrator: IblBakePmremIntegrator,
    diffuse_representation: IblBakeDiffuseRepresentation,
}

impl IblBakeRecipe {
    pub const fn algorithm_version(self) -> u64 {
        self.algorithm_version
    }

    pub const fn diffuse_source_face_size(self) -> u32 {
        self.diffuse_source_face_size
    }

    pub const fn irradiance_cube_face_size(self) -> u32 {
        self.irradiance_cube_face_size
    }

    pub const fn runtime_diffuse_sample_count(self) -> u32 {
        self.runtime_diffuse_sample_count
    }

    pub const fn full_roughness_cosine_threshold(self) -> Real {
        self.full_roughness_cosine_threshold
    }

    pub const fn fis_solid_angle_texel_scale(self) -> Real {
        self.fis_solid_angle_texel_scale
    }

    pub const fn roughest_mip_offset(self) -> Real {
        self.roughest_mip_offset
    }

    pub const fn roughness_mip_scale(self) -> Real {
        self.roughness_mip_scale
    }

    pub const fn output_format(self) -> IblBakeOutputFormat {
        self.output_format
    }

    pub const fn pmrem_integrator(self) -> IblBakePmremIntegrator {
        self.pmrem_integrator
    }

    pub const fn diffuse_representation(self) -> IblBakeDiffuseRepresentation {
        self.diffuse_representation
    }

    pub const fn asset_recipe_identity(self) -> IblBakeRecipeIdentity {
        self.identity(IblBakeDiffuseIntegrator::AssetImporterCpuSolidAngle)
    }

    pub const fn runtime_recipe_identity(self) -> IblBakeRecipeIdentity {
        self.identity(IblBakeDiffuseIntegrator::RendererGpuRuntimeHammersley)
    }

    pub const fn identity(
        self,
        diffuse_integrator: IblBakeDiffuseIntegrator,
    ) -> IblBakeRecipeIdentity {
        IblBakeRecipeIdentity::new(
            self.algorithm_version,
            self.pmrem_integrator,
            diffuse_integrator,
            self.output_format,
        )
    }

    /// Chooses the first source mip no wider than the recipe's diffuse input face.
    pub const fn diffuse_source_mip_level(self, face_size: u32, mip_count: u32) -> u32 {
        let face_size = if face_size == 0 { 1 } else { face_size };
        let mip_count = if mip_count == 0 { 1 } else { mip_count };
        let mut mip_level = 0;
        while mip_level + 1 < mip_count && face_size >> mip_level > self.diffuse_source_face_size {
            mip_level += 1;
        }
        mip_level
    }

    pub const fn pmrem_sample_count(self, roughness: Real, mip_level: u32) -> u32 {
        if mip_level == 0 || roughness < self.pmrem_low_roughness_threshold {
            self.pmrem_fast_sample_count
        } else if roughness >= self.pmrem_high_roughness_threshold {
            self.pmrem_rough_sample_count
        } else {
            self.pmrem_normal_sample_count
        }
    }

    pub fn pmrem_mip_from_roughness(self, roughness: Real, mip_count: u32) -> Real {
        let max_mip = mip_count.max(1) as Real - 1.0;
        let roughness = roughness.clamp(0.0, 1.0);
        if roughness <= Real::EPSILON || max_mip <= 0.0 {
            return 0.0;
        }
        (max_mip - self.roughest_mip_offset + self.roughness_mip_scale * roughness.log2())
            .clamp(0.0, max_mip)
    }

    pub fn roughness_from_pmrem_mip(self, mip_level: u32, mip_count: u32) -> Real {
        let max_mip = mip_count.max(1).saturating_sub(1);
        if max_mip == 0 || mip_level == 0 {
            return 0.0;
        }
        let level_from_1x1 = max_mip.saturating_sub(mip_level.min(max_mip)) as Real;
        2.0_f32
            .powf((self.roughest_mip_offset - level_from_1x1) / self.roughness_mip_scale)
            .clamp(0.0, 1.0)
    }
}

pub const CANONICAL_IBL_BAKE_ALGORITHM_VERSION: u64 = 2026_08_09_0006;
pub const CANONICAL_IBL_BAKE_DIFFUSE_SOURCE_FACE_SIZE: u32 = 32;
pub const CANONICAL_IBL_BAKE_IRRADIANCE_CUBE_FACE_SIZE: u32 = 32;
pub const CANONICAL_IBL_BAKE_ROUGHEST_MIP_OFFSET: Real = 1.0;
pub const CANONICAL_IBL_BAKE_ROUGHNESS_MIP_SCALE: Real = 1.2;

pub const CANONICAL_IBL_BAKE_RECIPE: IblBakeRecipe = IblBakeRecipe {
    algorithm_version: CANONICAL_IBL_BAKE_ALGORITHM_VERSION,
    diffuse_source_face_size: CANONICAL_IBL_BAKE_DIFFUSE_SOURCE_FACE_SIZE,
    irradiance_cube_face_size: CANONICAL_IBL_BAKE_IRRADIANCE_CUBE_FACE_SIZE,
    pmrem_fast_sample_count: 32,
    pmrem_normal_sample_count: 64,
    pmrem_rough_sample_count: 128,
    runtime_diffuse_sample_count: 64,
    pmrem_low_roughness_threshold: 0.1,
    pmrem_high_roughness_threshold: 0.75,
    full_roughness_cosine_threshold: 0.99,
    fis_solid_angle_texel_scale: 2.0,
    roughest_mip_offset: CANONICAL_IBL_BAKE_ROUGHEST_MIP_OFFSET,
    roughness_mip_scale: CANONICAL_IBL_BAKE_ROUGHNESS_MIP_SCALE,
    output_format: IblBakeOutputFormat::Rgba16Float,
    pmrem_integrator: IblBakePmremIntegrator::GgxFilteredImportanceSampling,
    diffuse_representation: IblBakeDiffuseRepresentation::Sh9WithOptionalIrradianceCube,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_recipe_selects_the_first_mip_at_or_below_its_diffuse_face_size() {
        assert_eq!(
            CANONICAL_IBL_BAKE_RECIPE.diffuse_source_mip_level(256, 9),
            3
        );
        assert_eq!(CANONICAL_IBL_BAKE_RECIPE.diffuse_source_mip_level(32, 6), 0);
        assert_eq!(CANONICAL_IBL_BAKE_RECIPE.diffuse_source_mip_level(16, 5), 0);
    }

    #[test]
    fn canonical_recipe_never_selects_a_mip_outside_the_declared_chain() {
        assert_eq!(
            CANONICAL_IBL_BAKE_RECIPE.diffuse_source_mip_level(256, 0),
            0
        );
        assert_eq!(CANONICAL_IBL_BAKE_RECIPE.diffuse_source_mip_level(1, 8), 0);
    }

    #[test]
    fn canonical_recipe_owns_cpu_and_gpu_sampling_policy() {
        let recipe = CANONICAL_IBL_BAKE_RECIPE;

        assert_eq!(recipe.pmrem_sample_count(0.0, 0), 32);
        assert_eq!(recipe.pmrem_sample_count(0.5, 4), 64);
        assert_eq!(recipe.pmrem_sample_count(0.75, 6), 128);
        assert_eq!(recipe.runtime_diffuse_sample_count(), 64);
        assert_eq!(recipe.full_roughness_cosine_threshold(), 0.99);
        assert_eq!(recipe.fis_solid_angle_texel_scale(), 2.0);
    }

    #[test]
    fn canonical_recipe_keeps_cpu_and_runtime_integrator_identities_distinct() {
        let asset = CANONICAL_IBL_BAKE_RECIPE.asset_recipe_identity();
        let runtime = CANONICAL_IBL_BAKE_RECIPE.runtime_recipe_identity();

        assert_ne!(asset, runtime);
        assert_eq!(asset.algorithm_version(), runtime.algorithm_version());
        assert_eq!(asset.pmrem_integrator(), runtime.pmrem_integrator());
        assert_eq!(asset.output_format(), IblBakeOutputFormat::Rgba16Float);
        assert_eq!(
            CANONICAL_IBL_BAKE_RECIPE.diffuse_representation(),
            IblBakeDiffuseRepresentation::Sh9WithOptionalIrradianceCube
        );
    }

    #[test]
    fn canonical_recipe_owns_the_bidirectional_roughness_mapping() {
        let recipe = CANONICAL_IBL_BAKE_RECIPE;
        for mip_level in 1..7 {
            let roughness = recipe.roughness_from_pmrem_mip(mip_level, 8);
            let resolved_mip = recipe.pmrem_mip_from_roughness(roughness, 8);
            assert!((resolved_mip - mip_level as Real).abs() <= 0.0001);
        }
    }
}

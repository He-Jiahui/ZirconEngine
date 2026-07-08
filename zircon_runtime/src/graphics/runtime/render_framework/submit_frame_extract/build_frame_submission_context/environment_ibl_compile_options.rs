use crate::asset::artifact::resolve_ibl_bake_artifact_runtime_dispatch;
use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    IblBakeArtifactContents, RenderFrameExtract, RenderFrameworkError,
};
use crate::graphics::RenderPipelineCompileOptions;

pub(super) fn compile_options_with_environment_ibl_bake_request(
    asset_manager: &ProjectAssetManager,
    extract: &RenderFrameExtract,
    options: RenderPipelineCompileOptions,
) -> Result<RenderPipelineCompileOptions, RenderFrameworkError> {
    let Some(request) = extract
        .environment
        .source_cubemap_ibl_bake_request(IblBakeArtifactContents::PMREM_SH9_IEM)
    else {
        return Ok(options.without_environment_ibl_bake_request());
    };
    let Some(store) = asset_manager.ibl_bake_artifact_cache_store() else {
        return Ok(options.without_environment_ibl_bake_request());
    };
    let dispatch = resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[])
        .map_err(|error| RenderFrameworkError::Backend(error.to_string()))?;
    if dispatch.requires_runtime_compute() {
        Ok(options.with_environment_ibl_bake_request(request))
    } else {
        Ok(options.without_environment_ibl_bake_request())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::asset::artifact::IblBakeArtifactCacheStore;
    use crate::asset::pipeline::manager::AssetManager;
    use crate::asset::project::{ProjectManifest, ProjectPaths};
    use crate::asset::{AssetUri, ProjectAssetManager};
    use crate::core::framework::render::{
        build_source_cubemap_from_equirect, EnvironmentExtract, FallbackSkyboxKind,
        IblBakeArtifactBlob, IblBakeArtifactDescriptor, IblBakeArtifactReadbackSections,
        PreviewEnvironmentExtract, RenderFrameExtract, RenderSceneGeometryExtract,
        RenderWorldSnapshotHandle, SceneViewportRenderPacket, SourceCubemapEnvironment,
        IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES, IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES,
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
    };
    use crate::core::math::Vec4;
    use crate::graphics::RenderPipelineCompileOptions;

    use super::compile_options_with_environment_ibl_bake_request;

    #[test]
    fn source_cubemap_cache_miss_enables_environment_ibl_bake_compile_request() {
        let root = unique_temp_project_root("ibl-compile-options-miss");
        let manager = project_asset_manager_with_root(&root);
        let extract = extract_with_source_cubemap();

        let options = compile_options_with_environment_ibl_bake_request(
            &manager,
            &extract,
            RenderPipelineCompileOptions::default(),
        )
        .expect("cache miss should resolve dispatch");

        assert!(options.environment_ibl_bake_request().is_some());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn source_cubemap_cache_hit_keeps_environment_ibl_bake_out_of_graph_key() {
        let root = unique_temp_project_root("ibl-compile-options-hit");
        let manager = project_asset_manager_with_root(&root);
        let extract = extract_with_source_cubemap();
        let request = extract
            .environment
            .source_cubemap_ibl_bake_request(
                crate::core::framework::render::IblBakeArtifactContents::PMREM_SH9_IEM,
            )
            .expect("source cubemap request");
        let store = IblBakeArtifactCacheStore::new(
            manager
                .current_project_manager()
                .expect("project")
                .paths()
                .runtime_cache_root(),
        );
        store
            .write_runtime_cache(&blob_for_request(&request))
            .expect("runtime cache seed should write");

        let options = compile_options_with_environment_ibl_bake_request(
            &manager,
            &extract,
            RenderPipelineCompileOptions::default().with_environment_ibl_bake_request(request),
        )
        .expect("cache hit should resolve dispatch");

        assert!(options.environment_ibl_bake_request().is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn disabled_environment_does_not_enable_ibl_bake_compile_request() {
        let root = unique_temp_project_root("ibl-compile-options-disabled");
        let manager = project_asset_manager_with_root(&root);
        let extract = RenderFrameExtract {
            environment: EnvironmentExtract::default(),
            ..RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(1),
                empty_packet(EnvironmentExtract::default()),
            )
        };

        let options = compile_options_with_environment_ibl_bake_request(
            &manager,
            &extract,
            RenderPipelineCompileOptions::default(),
        )
        .expect("disabled environment should not touch cache");

        assert!(options.environment_ibl_bake_request().is_none());
        let _ = fs::remove_dir_all(root);
    }

    fn project_asset_manager_with_root(root: &PathBuf) -> ProjectAssetManager {
        let paths = ProjectPaths::from_root(root).expect("project paths");
        paths.ensure_layout().expect("project layout");
        ProjectManifest::new(
            "ibl-compile-options",
            AssetUri::parse("res://scenes/main.scene.toml").expect("default scene uri"),
            1,
        )
        .save(paths.manifest_path())
        .expect("project manifest");
        let manager = ProjectAssetManager::default();
        AssetManager::open_project(&manager, root.to_string_lossy().as_ref())
            .expect("project open");
        manager
    }

    fn extract_with_source_cubemap() -> RenderFrameExtract {
        let environment = EnvironmentExtract::source_cubemap(SourceCubemapEnvironment::new(
            build_source_cubemap_from_equirect(8, |_, _| [0.25, 0.5, 0.75, 1.0]),
            7,
            [10, 20, 30, 40],
        ));
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            empty_packet(environment),
        )
    }

    fn empty_packet(environment: EnvironmentExtract) -> SceneViewportRenderPacket {
        SceneViewportRenderPacket {
            scene: RenderSceneGeometryExtract {
                camera: Default::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment,
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        }
    }

    fn blob_for_request(
        request: &crate::core::framework::render::IblBakeArtifactRequest,
    ) -> IblBakeArtifactBlob {
        let descriptor = IblBakeArtifactDescriptor::current(
            request.bake_key(),
            request.face_size(),
            request.mip_count(),
            request.required_contents(),
        );
        let readback = IblBakeArtifactReadbackSections::new(descriptor)
            .with_pmrem_rgba16f_bytes(vec![
                0;
                crate::core::framework::render::source_cubemap_sample_count(
                    request.face_size(),
                    request.mip_count(),
                )
                    * IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES
            ])
            .with_irradiance_sh9_bytes(vec![0; IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES])
            .with_irradiance_cube_rgba16f_bytes(vec![
                0;
                6 * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
                    * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE
                        as usize
                    * IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES
            ]);
        IblBakeArtifactBlob::from_payload(
            readback
                .into_payload()
                .expect("seed readback sections should assemble"),
        )
    }

    fn unique_temp_project_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("zircon-{name}-{nanos}"))
    }
}

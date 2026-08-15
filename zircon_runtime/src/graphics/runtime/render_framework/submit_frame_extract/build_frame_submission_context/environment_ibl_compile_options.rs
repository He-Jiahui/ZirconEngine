use std::sync::{Arc, Mutex};

use crate::asset::artifact::resolve_ibl_bake_artifact_runtime_dispatch;
use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    source_cubemap_environment_with_bake_artifact, IblBakeArtifactContents, RenderFrameExtract,
    RenderFrameworkError,
};
use crate::graphics::runtime::render_framework::render_framework_state::EnvironmentIblHydrationCache;
use crate::graphics::RenderPipelineCompileOptions;

pub(super) enum EnvironmentIblCacheResolution {
    RuntimeComputeRequired(crate::graphics::EnvironmentIblBakeReservation),
    RuntimeBakePending,
    Hydrated,
    RuntimeBakeUnavailable,
}

impl EnvironmentIblCacheResolution {
    const fn requires_runtime_compute(&self) -> bool {
        matches!(self, Self::RuntimeComputeRequired(_))
    }

    pub(super) fn take_runtime_bake_reservation(
        self,
    ) -> Option<crate::graphics::EnvironmentIblBakeReservation> {
        match self {
            Self::RuntimeComputeRequired(reservation) => Some(reservation),
            Self::RuntimeBakePending | Self::Hydrated | Self::RuntimeBakeUnavailable => None,
        }
    }
}

pub(super) fn resolve_and_rehydrate_environment_ibl_cache(
    asset_manager: &ProjectAssetManager,
    hydration_cache: &Arc<Mutex<EnvironmentIblHydrationCache>>,
    extract: &mut RenderFrameExtract,
) -> Result<Option<EnvironmentIblCacheResolution>, RenderFrameworkError> {
    let Some(request) = extract
        .environment
        .source_cubemap_ibl_bake_request(IblBakeArtifactContents::PMREM_SH9)
    else {
        return Ok(None);
    };
    let Some(environment) = extract.environment.skybox.source_cubemap.as_ref() else {
        return Ok(None);
    };
    if environment
        .accepted_bake_artifact_descriptor()
        .is_some_and(|descriptor| descriptor.is_current_for(&request))
    {
        hydration_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear_pending_runtime_bake(&request);
        return Ok(Some(EnvironmentIblCacheResolution::Hydrated));
    }
    let cached = hydration_cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&request, environment);
    if let Some(cached) = cached {
        extract.environment.skybox.source_cubemap = Some(cached);
        return Ok(Some(EnvironmentIblCacheResolution::Hydrated));
    }
    let Some(store) = asset_manager.ibl_bake_artifact_cache_store() else {
        return Ok(Some(EnvironmentIblCacheResolution::RuntimeBakeUnavailable));
    };
    let dispatch = resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[])
        .map_err(|error| RenderFrameworkError::Backend(error.to_string()))?;
    let Some(payload) = dispatch.payload() else {
        let resolution =
            EnvironmentIblHydrationCache::reserve_runtime_bake(hydration_cache, request)
                .map(EnvironmentIblCacheResolution::RuntimeComputeRequired)
                .unwrap_or(EnvironmentIblCacheResolution::RuntimeBakePending);
        return Ok(Some(resolution));
    };
    let hydrated =
        source_cubemap_environment_with_bake_artifact(environment, payload).map_err(|error| {
            RenderFrameworkError::Backend(format!("rehydrate environment IBL artifact: {error:?}"))
        })?;
    hydration_cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(request, hydrated.clone());
    extract.environment.skybox.source_cubemap = Some(hydrated);
    Ok(Some(EnvironmentIblCacheResolution::Hydrated))
}

pub(super) fn compile_options_with_environment_ibl_bake_request(
    extract: &RenderFrameExtract,
    options: RenderPipelineCompileOptions,
    resolution: Option<&EnvironmentIblCacheResolution>,
) -> Result<RenderPipelineCompileOptions, RenderFrameworkError> {
    let Some(request) = extract
        .environment
        .source_cubemap_ibl_bake_request(IblBakeArtifactContents::PMREM_SH9)
    else {
        return Ok(options.without_environment_ibl_bake_request());
    };
    let Some(resolution) = resolution else {
        return Ok(options.without_environment_ibl_bake_request());
    };
    if resolution.requires_runtime_compute() {
        Ok(options.with_environment_ibl_bake_request(request))
    } else {
        Ok(options.without_environment_ibl_bake_request())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::asset::artifact::IblBakeArtifactCacheStore;
    use crate::asset::pipeline::manager::AssetManager;
    use crate::asset::project::{ProjectManifest, ProjectPaths};
    use crate::asset::{AssetUri, ProjectAssetManager};
    use crate::core::framework::render::{
        build_source_cubemap_from_equirect, source_cubemap_environment_with_bake_artifact,
        EnvironmentExtract, FallbackSkyboxKind, IblBakeArtifactBlob, IblBakeArtifactContents,
        IblBakeArtifactDescriptor, IblBakeArtifactPayload, IblBakeArtifactReadbackSections,
        PreviewEnvironmentExtract, RenderFrameExtract, RenderSceneGeometryExtract,
        RenderWorldSnapshotHandle, SceneViewportRenderPacket, SourceCubemapEnvironment,
        SourceCubemapIrradianceCube, IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES,
        IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES,
    };
    use crate::core::math::Vec4;
    use crate::graphics::runtime::render_framework::render_framework_state::EnvironmentIblHydrationCache;
    use crate::graphics::RenderPipelineCompileOptions;

    use super::{
        compile_options_with_environment_ibl_bake_request,
        resolve_and_rehydrate_environment_ibl_cache,
    };

    #[test]
    fn source_cubemap_cache_miss_enables_environment_ibl_bake_compile_request() {
        let root = unique_temp_project_root("ibl-compile-options-miss");
        let manager = project_asset_manager_with_root(&root);
        let hydration_cache = Arc::new(Mutex::<EnvironmentIblHydrationCache>::default());
        let mut extract = extract_with_source_cubemap();
        let resolution =
            resolve_and_rehydrate_environment_ibl_cache(&manager, &hydration_cache, &mut extract)
                .expect("cache miss should resolve dispatch");

        let options = compile_options_with_environment_ibl_bake_request(
            &extract,
            RenderPipelineCompileOptions::default(),
            resolution.as_ref(),
        )
        .expect("cache miss should resolve dispatch");

        assert!(options.environment_ibl_bake_request().is_some());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn accepted_cpu_artifact_prevents_runtime_fallback_and_retains_iem() {
        let manager = ProjectAssetManager::default();
        let hydration_cache = Arc::new(Mutex::<EnvironmentIblHydrationCache>::default());
        let mut extract = extract_with_source_cubemap();
        let source = extract
            .environment
            .skybox
            .source_cubemap
            .as_ref()
            .expect("source cubemap")
            .clone();
        let request = source.ibl_bake_artifact_request(IblBakeArtifactContents::PMREM_SH9_IEM);
        let irradiance_cube = SourceCubemapIrradianceCube::new(1, vec![[0.1, 0.2, 0.3]; 6]);
        let payload = IblBakeArtifactPayload::from_source_cubemap(
            IblBakeArtifactDescriptor::current_for_request(&request),
            &source.mip_chain,
            Some(&irradiance_cube),
        )
        .expect("CPU artifact payload");
        extract.environment.skybox.source_cubemap = Some(
            source_cubemap_environment_with_bake_artifact(&source, &payload)
                .expect("CPU artifact should hydrate source environment"),
        );

        let resolution =
            resolve_and_rehydrate_environment_ibl_cache(&manager, &hydration_cache, &mut extract)
                .expect("accepted CPU artifact should resolve without a runtime cache store");
        let options = compile_options_with_environment_ibl_bake_request(
            &extract,
            RenderPipelineCompileOptions::default(),
            resolution.as_ref(),
        )
        .expect("accepted CPU artifact should keep bake out of graph");

        assert!(options.environment_ibl_bake_request().is_none());
        let hydrated = extract
            .environment
            .skybox
            .source_cubemap_environment()
            .expect("accepted source environment");
        assert_eq!(
            hydrated.accepted_bake_artifact_descriptor(),
            Some(payload.descriptor())
        );
        assert_eq!(hydrated.irradiance_cube(), Some(&irradiance_cube));
    }

    #[test]
    fn pending_runtime_bake_keeps_repeated_cache_miss_out_of_graph_key() {
        let root = unique_temp_project_root("ibl-compile-options-pending");
        let manager = project_asset_manager_with_root(&root);
        let hydration_cache = Arc::new(Mutex::<EnvironmentIblHydrationCache>::default());
        let mut first_extract = extract_with_source_cubemap();
        let first_resolution = resolve_and_rehydrate_environment_ibl_cache(
            &manager,
            &hydration_cache,
            &mut first_extract,
        )
        .expect("first cache miss should schedule a runtime bake");
        let first_options = compile_options_with_environment_ibl_bake_request(
            &first_extract,
            RenderPipelineCompileOptions::default(),
            first_resolution.as_ref(),
        )
        .expect("first cache miss should compile the bake graph");
        assert!(first_options.environment_ibl_bake_request().is_some());

        let mut repeated_extract = extract_with_source_cubemap();
        let repeated_resolution = resolve_and_rehydrate_environment_ibl_cache(
            &manager,
            &hydration_cache,
            &mut repeated_extract,
        )
        .expect("pending runtime bake should resolve without a second graph request");
        let repeated_options = compile_options_with_environment_ibl_bake_request(
            &repeated_extract,
            RenderPipelineCompileOptions::default(),
            repeated_resolution.as_ref(),
        )
        .expect("pending runtime bake should stay out of graph key");

        assert!(repeated_options.environment_ibl_bake_request().is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn dropped_runtime_bake_reservation_allows_the_next_frame_to_retry() {
        let root = unique_temp_project_root("ibl-compile-options-retry");
        let manager = project_asset_manager_with_root(&root);
        let hydration_cache = Arc::new(Mutex::<EnvironmentIblHydrationCache>::default());
        let mut failed_build_extract = extract_with_source_cubemap();
        let failed_build_resolution = resolve_and_rehydrate_environment_ibl_cache(
            &manager,
            &hydration_cache,
            &mut failed_build_extract,
        )
        .expect("first cache miss should reserve one runtime bake");
        let failed_build_options = compile_options_with_environment_ibl_bake_request(
            &failed_build_extract,
            RenderPipelineCompileOptions::default(),
            failed_build_resolution.as_ref(),
        )
        .expect("first cache miss should request the bake graph");
        assert!(failed_build_options
            .environment_ibl_bake_request()
            .is_some());

        // A later pipeline-build error drops this resolution before it reaches the writeback queue.
        drop(failed_build_resolution);

        let mut retry_extract = extract_with_source_cubemap();
        let retry_resolution = resolve_and_rehydrate_environment_ibl_cache(
            &manager,
            &hydration_cache,
            &mut retry_extract,
        )
        .expect("a dropped reservation must permit the next frame to retry");
        let retry_options = compile_options_with_environment_ibl_bake_request(
            &retry_extract,
            RenderPipelineCompileOptions::default(),
            retry_resolution.as_ref(),
        )
        .expect("retry should request a bake graph after the failed build");

        assert!(retry_options.environment_ibl_bake_request().is_some());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn source_cubemap_cache_hit_keeps_environment_ibl_bake_out_of_graph_key() {
        let root = unique_temp_project_root("ibl-compile-options-hit");
        let manager = project_asset_manager_with_root(&root);
        let hydration_cache = Arc::new(Mutex::<EnvironmentIblHydrationCache>::default());
        let mut extract = extract_with_source_cubemap();
        let request = extract
            .environment
            .source_cubemap_ibl_bake_request(
                crate::core::framework::render::IblBakeArtifactContents::PMREM_SH9,
            )
            .expect("source cubemap request");
        let store = IblBakeArtifactCacheStore::new(
            manager
                .current_project_manager()
                .expect("project")
                .paths()
                .cache_root(),
        );
        store
            .write_runtime_cache(&blob_for_request(&request))
            .expect("runtime cache seed should write");
        let resolution =
            resolve_and_rehydrate_environment_ibl_cache(&manager, &hydration_cache, &mut extract)
                .expect("cache hit should rehydrate source cubemap environment");

        let options = compile_options_with_environment_ibl_bake_request(
            &extract,
            RenderPipelineCompileOptions::default().with_environment_ibl_bake_request(request),
            resolution.as_ref(),
        )
        .expect("cache hit should resolve dispatch");

        assert!(options.environment_ibl_bake_request().is_none());
        let source_environment = extract
            .environment
            .skybox
            .source_cubemap_environment()
            .expect("source cubemap environment should remain enabled");
        assert_ne!(source_environment.bake_artifact_hash, [0; 4]);
        assert!(source_environment.prepared_upload_artifact().is_some());

        fs::remove_dir_all(&root).expect("remove backing cache after first hydration");
        let mut repeated_extract = extract_with_source_cubemap();
        let repeated_source = repeated_extract
            .environment
            .skybox
            .source_cubemap
            .as_mut()
            .expect("source cubemap");
        repeated_source.intensity = 2.5;
        repeated_source.rotation_radians = 0.75;
        let repeated_resolution = resolve_and_rehydrate_environment_ibl_cache(
            &manager,
            &hydration_cache,
            &mut repeated_extract,
        )
        .expect("memoized hydration must not require the removed cache file");
        assert!(matches!(
            repeated_resolution,
            Some(super::EnvironmentIblCacheResolution::Hydrated)
        ));
        let repeated_source = repeated_extract
            .environment
            .skybox
            .source_cubemap_environment()
            .expect("memoized source cubemap");
        assert_eq!(repeated_source.intensity, 2.5);
        assert_eq!(repeated_source.rotation_radians, 0.75);
        assert!(repeated_source.prepared_upload_artifact().is_some());
    }

    #[test]
    fn source_cubemap_without_project_cache_keeps_runtime_bake_out_of_two_frame_graph_keys() {
        let manager = ProjectAssetManager::default();
        let hydration_cache = Arc::new(Mutex::<EnvironmentIblHydrationCache>::default());
        let mut first_extract = extract_with_source_cubemap();
        let first_resolution = resolve_and_rehydrate_environment_ibl_cache(
            &manager,
            &hydration_cache,
            &mut first_extract,
        )
        .expect("missing cache store should use the source environment without a bake graph");
        let first_options = compile_options_with_environment_ibl_bake_request(
            &first_extract,
            RenderPipelineCompileOptions::default(),
            first_resolution.as_ref(),
        )
        .expect("missing cache store should suppress the unused bake graph");
        assert!(first_options.environment_ibl_bake_request().is_none());

        let mut repeated_extract = extract_with_source_cubemap();
        let repeated_resolution = resolve_and_rehydrate_environment_ibl_cache(
            &manager,
            &hydration_cache,
            &mut repeated_extract,
        )
        .expect("second frame without a cache store should remain graph-free");
        let repeated_options = compile_options_with_environment_ibl_bake_request(
            &repeated_extract,
            RenderPipelineCompileOptions::default(),
            repeated_resolution.as_ref(),
        )
        .expect("second frame without a cache store should suppress the unused bake graph");

        assert!(repeated_options.environment_ibl_bake_request().is_none());
    }

    #[test]
    fn disabled_environment_does_not_enable_ibl_bake_compile_request() {
        let root = unique_temp_project_root("ibl-compile-options-disabled");
        let manager = project_asset_manager_with_root(&root);
        let hydration_cache = Arc::new(Mutex::<EnvironmentIblHydrationCache>::default());
        let mut extract = RenderFrameExtract {
            environment: EnvironmentExtract::default(),
            ..RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(1),
                empty_packet(EnvironmentExtract::default()),
            )
        };
        let resolution =
            resolve_and_rehydrate_environment_ibl_cache(&manager, &hydration_cache, &mut extract)
                .expect("disabled environment should not touch cache");

        let options = compile_options_with_environment_ibl_bake_request(
            &extract,
            RenderPipelineCompileOptions::default(),
            resolution.as_ref(),
        )
        .expect("disabled environment should not touch cache");

        assert!(options.environment_ibl_bake_request().is_none());
        let _ = fs::remove_dir_all(root);
    }

    fn project_asset_manager_with_root(root: &PathBuf) -> ProjectAssetManager {
        let paths = ProjectPaths::from_root(root).expect("project paths");
        paths
            .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
            .expect("project layout");
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
        let descriptor = IblBakeArtifactDescriptor::current_for_runtime_cache_request(request);
        let readback = IblBakeArtifactReadbackSections::new(descriptor)
            .with_pmrem_rgba16f_bytes(vec![
                0;
                crate::core::framework::render::source_cubemap_sample_count(
                    request.pmrem_face_size(),
                    request.pmrem_mip_count(),
                )
                    * IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES
            ])
            .with_irradiance_sh9_bytes(vec![0; IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES]);
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

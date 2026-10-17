//! Shared real WGPU frame fixture for rendering feature runtime product smokes.
#[path = "project.rs"]
mod product_project;

use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::{ProjectAssetManager, ProjectAssetManagerAccess};
use zircon_runtime::core::framework::render::{
    CapturedFrame, GeometryExtract, RenderFrameExtract, RenderFramework, RenderMeshSnapshot,
    RenderQualityProfile, RenderStats, RenderViewportDescriptor, RenderWorldSnapshotHandle,
    RendererCommon,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::manager::{manager_service_handle, RegisteredManagerService};
use zircon_runtime::core::math::{Transform, UVec2, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
};
use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderPassExecutorRegistration, RuntimePrepareCollectorRegistration,
    WgpuRenderFramework,
};

const ASSET_MODULE: &str = "RenderingFeatureSmokeAssetRuntime";
const ASSET_SERVICE: &str = "RenderingFeatureSmokeAssetRuntime.Manager.ProjectAssetManager";
const VIEWPORT_SIZE: UVec2 = UVec2::new(96, 72);

struct SmokeRuntime {
    runtime: CoreRuntime,
    access: ProjectAssetManagerAccess,
}

impl SmokeRuntime {
    fn new() -> Self {
        let manager = Arc::new(ProjectAssetManager::default());
        product_project::open_smoke_project(&manager, "render_feature_smoke");
        let runtime = CoreRuntime::new();
        runtime
            .register_module(
                ModuleDescriptor::new(ASSET_MODULE, "render feature smoke assets").with_manager(
                    ManagerDescriptor::new(
                        RegistryName::from_parts(
                            ASSET_MODULE,
                            ServiceKind::Manager,
                            "ProjectAssetManager",
                        ),
                        StartupMode::Immediate,
                        Vec::new(),
                        Arc::new(move |_| {
                            Ok(
                                Arc::new(RegisteredManagerService::new(Arc::clone(&manager)))
                                    as ServiceObject,
                            )
                        }),
                    ),
                ),
            )
            .expect("render feature smoke asset module should register");
        runtime
            .activate_module(ASSET_MODULE)
            .expect("render feature smoke asset module should activate");
        let core = runtime.handle();
        let handle = manager_service_handle(&core, ASSET_SERVICE)
            .expect("render feature smoke asset manager should resolve");
        Self {
            runtime,
            access: ProjectAssetManagerAccess::new(core, handle),
        }
    }

    fn worker_pool(&self) -> zircon_runtime::core::TaskPool {
        self.runtime.task_graph().worker_pool().clone()
    }
}

/// Submit and capture one nonempty builtin-cube scene with the supplied production feature graph.
pub fn render_nonempty_product_frame(
    features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
) -> (CapturedFrame, RenderStats) {
    render_nonempty_product_frame_with_prepare(features, executors, [], |_| {})
}

/// Adds real feature-owned preparation and extract configuration without substituting the
/// product render/capture path used by the original feature smokes.
pub fn render_nonempty_product_frame_with_prepare(
    features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    collectors: impl IntoIterator<Item = RuntimePrepareCollectorRegistration>,
    configure: impl FnOnce(&mut RenderFrameExtract),
) -> (CapturedFrame, RenderStats) {
    let runtime = SmokeRuntime::new();
    let framework = WgpuRenderFramework::new_with_plugin_render_extensions(
        runtime.access.clone(),
        features,
        executors,
        collectors,
        [],
        [],
        runtime.worker_pool(),
    )
    .expect("feature smoke WGPU framework should initialize");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(VIEWPORT_SIZE))
        .expect("feature smoke viewport should initialize");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("render-feature-product-smoke")
                .with_screen_space_ambient_occlusion(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_anti_alias(false),
        )
        .expect("feature smoke profile should apply");

    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(82_001),
        zircon_runtime::scene::world::World::new().to_render_snapshot(),
    );
    *extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![RenderMeshSnapshot {
            node_id: 82_002,
            stable_instance_key: 82_002 << 16,
            transform_revision: 0,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "builtin://cube",
            )),
            mesh: None,
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "builtin://material/default",
            )),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            common: RendererCommon::default(),
        }],
    );
    configure(&mut extract);
    framework
        .submit_frame_extract(viewport, extract)
        .expect("feature smoke scene should submit");
    let frame = framework
        .capture_frame(viewport)
        .expect("feature smoke frame capture should complete")
        .expect("feature smoke frame should have captured pixels");
    assert_eq!(
        (frame.width, frame.height),
        (VIEWPORT_SIZE.x, VIEWPORT_SIZE.y)
    );
    assert_eq!(
        frame.rgba.len(),
        (VIEWPORT_SIZE.x * VIEWPORT_SIZE.y * 4) as usize
    );
    let stats = framework
        .query_stats()
        .expect("feature smoke stats should be available");
    assert!(stats.last_mesh_draw_count > 0, "builtin cube should draw");
    assert!(
        !stats.last_graph_executed_passes.is_empty(),
        "feature smoke should execute the host graph"
    );
    framework
        .destroy_viewport(viewport)
        .expect("feature smoke viewport should be released");
    (frame, stats)
}

use std::path::Path;

use thiserror::Error;
use zircon_runtime::asset::artifact::{
    IblSourceCubemapStagedBundleReport, IblSourceCubemapStagingStore,
};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render::{
    build_source_cubemap_from_captured_faces_with_quality, source_cubemap_capture_hash,
    RenderOverlayExtract, RenderSceneSnapshot,
};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::graphics::{GraphicsError, SceneRenderer};

use super::{
    ReflectionProbeCaptureRequest, ReflectionProbeCaptureRequestError,
    REFLECTION_PROBE_CAPTURE_FACE_VIEWS,
};

#[derive(Debug)]
pub struct ReflectionProbeCaptureReport {
    pub captured_face_count: usize,
    pub source_hash: [u32; 4],
    pub staged_bundle: IblSourceCubemapStagedBundleReport,
}

pub fn capture_and_persist_reflection_probe(
    renderer: &mut SceneRenderer,
    scene: &RenderSceneSnapshot,
    library_root: impl AsRef<Path>,
    request: &ReflectionProbeCaptureRequest,
) -> Result<ReflectionProbeCaptureReport, ReflectionProbeCaptureError> {
    request.validate()?;
    let face_size = request.face_size;
    let mut captured_face_texels = Vec::with_capacity(
        face_size as usize * face_size as usize * REFLECTION_PROBE_CAPTURE_FACE_VIEWS.len(),
    );

    for face_view in REFLECTION_PROBE_CAPTURE_FACE_VIEWS {
        let mut face_scene = scene.clone();
        face_scene.scene.camera =
            face_view.camera(request.position, request.near_plane, request.far_plane);
        face_scene.overlays = RenderOverlayExtract::default();
        face_scene.virtual_geometry_debug = None;
        let mut face_texels =
            renderer.render_scene_color_hdr(face_scene, UVec2::splat(face_size))?;
        face_view.transform_to_cmft_layout(face_size, &mut face_texels);
        captured_face_texels.extend(face_texels);
    }

    let source_hash = source_cubemap_capture_hash(face_size, &captured_face_texels);
    let cubemap = build_source_cubemap_from_captured_faces_with_quality(
        face_size,
        captured_face_texels,
        request.quality.source_prefilter_quality(),
    );
    let bake_request = request.ibl_bake_request(source_hash);
    let output_uri = AssetUri::parse(&request.output_uri)
        .map_err(|error| ReflectionProbeCaptureError::OutputUri(error.to_string()))?;
    let staged_bundle = IblSourceCubemapStagingStore::new(library_root.as_ref())
        .write_source_cubemap_staged_bundle(&bake_request, output_uri, &cubemap, None)
        .map_err(|error| ReflectionProbeCaptureError::Persist(error.to_string()))?;

    Ok(ReflectionProbeCaptureReport {
        captured_face_count: REFLECTION_PROBE_CAPTURE_FACE_VIEWS.len(),
        source_hash,
        staged_bundle,
    })
}

#[derive(Debug, Error)]
pub enum ReflectionProbeCaptureError {
    #[error(transparent)]
    InvalidRequest(#[from] ReflectionProbeCaptureRequestError),
    #[error(transparent)]
    Render(#[from] GraphicsError),
    #[error("invalid reflection-probe output URI: {0}")]
    OutputUri(String),
    #[error("persist reflection-probe .zcube/.zribl bundle: {0}")]
    Persist(String),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
    use zircon_runtime::core::framework::render::{
        EnvironmentExtract, PreviewEnvironmentExtract, RenderSceneGeometryExtract,
        SceneViewportRenderPacket,
    };
    use zircon_runtime::core::math::Vec4;

    use super::*;
    use crate::capture::ReflectionProbeCaptureQuality;

    #[test]
    #[ignore = "manual WGPU six-face reflection-probe capture product acceptance"]
    fn reflection_probe_capture_product_captures_six_hdr_faces_and_persists_zcube_and_zribl() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(5)
            .expect("reflection-probe runtime manifest should live below the workspace root");
        let output_root = workspace_root
            .join("docs")
            .join("tests")
            .join("runtime")
            .join("shader")
            .join("reflection_probe_capture_product_20260711");
        fs::create_dir_all(&output_root).expect("create reflection-probe product output root");

        let environment = EnvironmentExtract::procedural_default();
        let scene = SceneViewportRenderPacket {
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
            preview: PreviewEnvironmentExtract::from_environment(&environment, true, Vec4::ZERO),
            environment,
            virtual_geometry_debug: None,
        };
        let request = ReflectionProbeCaptureRequest::new(
            "probe-product-acceptance",
            "res://reflection-probes/product-acceptance.zcube",
            [0.0; 3],
            1,
        )
        .with_face_size(64)
        .with_quality(ReflectionProbeCaptureQuality::Fast);
        let mut renderer = SceneRenderer::new(Arc::new(ProjectAssetManager::default()))
            .expect("create WGPU scene renderer");

        let report =
            capture_and_persist_reflection_probe(&mut renderer, &scene, &output_root, &request)
                .expect("capture and persist six-face reflection probe");

        assert_eq!(report.captured_face_count, 6);
        assert_ne!(report.source_hash, [0; 4]);
        assert!(report.staged_bundle.source_zcube().path().is_file());
        assert!(report.staged_bundle.source_zcube().payload_len() > 0);
        assert!(report.staged_bundle.asset_derived().path().is_file());
        assert!(report.staged_bundle.asset_derived().payload_len() > 0);
        println!(
            "reflection-probe capture product: faces={}, source_hash={:08x?}, zcube={}, zribl={}",
            report.captured_face_count,
            report.source_hash,
            report.staged_bundle.source_zcube().path().display(),
            report.staged_bundle.asset_derived().path().display(),
        );
    }
}

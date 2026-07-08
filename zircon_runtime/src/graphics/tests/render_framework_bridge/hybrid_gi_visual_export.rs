use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::core::framework::render::{
    CapturedFrame, RenderFramework, RenderHybridGiReadbackOutputs, RenderPluginRendererOutputs,
    RenderQualityProfile, RenderViewportDescriptor,
};
use crate::graphics::runtime::WgpuRenderFramework;
use crate::graphics::{
    HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput, HybridGiRuntimePrepareOutput,
    HybridGiRuntimeProvider, HybridGiRuntimeProviderRegistration, HybridGiRuntimeState,
    HybridGiRuntimeUpdate, RenderPassExecutionContext, RenderPassExecutorRegistration,
};

use super::*;

const OUTPUT_STEM: &str = "plan18_hybrid_gi_lumen_style_seed_visual_20260707";
const PANEL_SEPARATOR_PX: u32 = 4;

#[test]
#[ignore = "writes Hybrid GI visual evidence under docs/tests/runtime/render"]
fn export_hybrid_gi_lumen_style_seed_visual_png() {
    let viewport_size = UVec2::new(160, 120);
    let baseline = render_hybrid_gi_visual_frame(viewport_size, None, "baseline");
    let warm = render_hybrid_gi_visual_frame(viewport_size, Some([255, 72, 48]), "warm");
    let cool = render_hybrid_gi_visual_frame(viewport_size, Some([48, 96, 255]), "cool");

    assert_same_extent(&baseline, &warm);
    assert_same_extent(&warm, &cool);
    assert_nonblank(&baseline, "baseline");
    assert_nonblank(&warm, "warm Hybrid GI seed");
    assert_nonblank(&cool, "cool Hybrid GI seed");

    let warm_red = center_channel_average(&warm, 0);
    let cool_red = center_channel_average(&cool, 0);
    let warm_blue = center_channel_average(&warm, 2);
    let cool_blue = center_channel_average(&cool, 2);

    assert!(
        warm_red > cool_red + 1.0,
        "expected warm HGI seed to raise center red channel; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 1.0,
        "expected cool HGI seed to raise center blue channel; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );

    let output_dir = render_test_output_dir();
    let png_path = output_dir.join(format!("{OUTPUT_STEM}.png"));
    let report_path = output_dir.join(format!("{OUTPUT_STEM}.txt"));

    write_three_panel_png(&png_path, [&baseline, &warm, &cool]);
    fs::write(
        &report_path,
        format!(
            "Hybrid GI Lumen-style seed visual export\n\
             output_png={}\n\
             viewport={}x{}\n\
             baseline_luma={:.2}\n\
             warm_luma={:.2}\n\
             cool_luma={:.2}\n\
             warm_red={warm_red:.2}\n\
             cool_red={cool_red:.2}\n\
             warm_blue={warm_blue:.2}\n\
             cool_blue={cool_blue:.2}\n",
            png_path.display(),
            warm.width,
            warm.height,
            average_luma(&baseline),
            average_luma(&warm),
            average_luma(&cool),
        ),
    )
    .expect("write Hybrid GI visual export report");
}

fn render_hybrid_gi_visual_frame(
    viewport_size: UVec2,
    probe_irradiance_rgb: Option<[u8; 3]>,
    profile_suffix: &str,
) -> CapturedFrame {
    let framework = hybrid_gi_visual_framework(probe_irradiance_rgb);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(viewport, hybrid_gi_visual_quality_profile(profile_suffix))
        .unwrap();
    framework
        .submit_frame_extract(
            viewport,
            direct_hybrid_gi_extract(viewport_size, probe_irradiance_rgb.unwrap_or([0, 0, 0])),
        )
        .unwrap();
    framework
        .capture_frame(viewport)
        .unwrap()
        .expect("Hybrid GI visual test frame should be capturable")
}

fn hybrid_gi_visual_framework(probe_irradiance_rgb: Option<[u8; 3]>) -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_extensions(
        Arc::new(ProjectAssetManager::default()),
        [direct_hybrid_gi_render_feature_descriptor()],
        direct_hybrid_gi_executor_registrations(),
        Vec::new(),
        [seeded_hybrid_gi_runtime_provider(probe_irradiance_rgb)],
        Vec::new(),
    )
    .unwrap()
}

fn hybrid_gi_visual_quality_profile(profile_suffix: &str) -> RenderQualityProfile {
    RenderQualityProfile::new(format!("hybrid-gi-visual-{profile_suffix}"))
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(false)
        .with_virtual_geometry(false)
        .with_hybrid_global_illumination(true)
        .with_async_compute(false)
}

fn direct_hybrid_gi_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "hybrid_gi",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-scene-prepare",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.scene-prepare")
            .read_texture("scene-depth")
            .write_buffer("hybrid-gi-scene"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-trace-schedule",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("hybrid-gi.trace-schedule")
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                "zircon-hybrid-gi-trace-schedule",
                [8, 8, 1],
                [1, 1, 1],
            ))
            .read_buffer("hybrid-gi-scene")
            .write_buffer("hybrid-gi-trace"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-resolve",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.resolve")
            .read_buffer("hybrid-gi-trace")
            .write_texture("hybrid-gi-lighting"),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::HybridGlobalIllumination)
}

fn direct_hybrid_gi_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    [
        "hybrid-gi.scene-prepare",
        "hybrid-gi.trace-schedule",
        "hybrid-gi.resolve",
    ]
    .into_iter()
    .map(|executor_id| RenderPassExecutorRegistration::new(executor_id, noop_render_pass_executor))
    .collect()
}

fn noop_render_pass_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Ok(())
}

fn direct_hybrid_gi_extract(viewport_size: UVec2, rt_lighting_rgb: [u8; 3]) -> RenderFrameExtract {
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![direct_hybrid_gi_probe(mesh, 200)],
        trace_regions: vec![direct_hybrid_gi_trace_region(mesh, 300, rt_lighting_rgb)],
    });
    extract
}

fn direct_hybrid_gi_probe(entity: u64, probe_id: u32) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity,
        probe_id,
        position: Vec3::ZERO,
        radius: 1.0,
        parent_probe_id: None,
        resident: true,
        ray_budget: 128,
    }
}

fn direct_hybrid_gi_trace_region(
    entity: u64,
    region_id: u32,
    rt_lighting_rgb: [u8; 3],
) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        entity,
        region_id,
        bounds_center: Vec3::ZERO,
        bounds_radius: 1.0,
        screen_coverage: 1.0,
        rt_lighting_rgb,
    }
}

fn seeded_hybrid_gi_runtime_provider(
    probe_irradiance_rgb: Option<[u8; 3]>,
) -> HybridGiRuntimeProviderRegistration {
    HybridGiRuntimeProviderRegistration::new(
        "test.hybrid-gi.visual-seed",
        Arc::new(SeededHybridGiRuntimeProvider {
            probe_irradiance_rgb,
        }),
    )
}

#[derive(Debug)]
struct SeededHybridGiRuntimeProvider {
    probe_irradiance_rgb: Option<[u8; 3]>,
}

impl HybridGiRuntimeProvider for SeededHybridGiRuntimeProvider {
    fn create_state(&self) -> Box<dyn HybridGiRuntimeState> {
        Box::new(SeededHybridGiRuntimeState {
            probe_irradiance_rgb: self.probe_irradiance_rgb,
        })
    }
}

struct SeededHybridGiRuntimeState {
    probe_irradiance_rgb: Option<[u8; 3]>,
}

impl HybridGiRuntimeState for SeededHybridGiRuntimeState {
    fn prepare_frame(
        &mut self,
        input: HybridGiRuntimePrepareInput<'_>,
    ) -> HybridGiRuntimePrepareOutput {
        let renderer_outputs = self
            .probe_irradiance_rgb
            .map(|rgb| RenderPluginRendererOutputs {
                hybrid_gi: seeded_hybrid_gi_readback_outputs(&input, rgb),
                ..RenderPluginRendererOutputs::default()
            })
            .unwrap_or_default();

        HybridGiRuntimePrepareOutput::new(Vec::new()).with_renderer_outputs(renderer_outputs)
    }

    fn update_after_render(&mut self, _feedback: HybridGiRuntimeFeedback) -> HybridGiRuntimeUpdate {
        HybridGiRuntimeUpdate::default()
    }
}

fn seeded_hybrid_gi_readback_outputs(
    input: &HybridGiRuntimePrepareInput<'_>,
    rgb: [u8; 3],
) -> RenderHybridGiReadbackOutputs {
    let extract = input.extract();
    let completed_probe_ids = extract
        .map(|extract| {
            extract
                .probes
                .iter()
                .map(|probe| probe.probe_id)
                .collect::<Vec<_>>()
        })
        .filter(|probe_ids| !probe_ids.is_empty())
        .unwrap_or_else(|| vec![200]);
    let completed_trace_region_ids = extract
        .map(|extract| {
            extract
                .trace_regions
                .iter()
                .map(|region| region.region_id)
                .collect::<Vec<_>>()
        })
        .filter(|region_ids| !region_ids.is_empty())
        .unwrap_or_else(|| vec![300]);
    let rgb16 = [u16::from(rgb[0]), u16::from(rgb[1]), u16::from(rgb[2])];

    RenderHybridGiReadbackOutputs {
        completed_probe_ids,
        completed_trace_region_ids,
        probe_irradiance_rgb: vec![rgb16],
        probe_rt_lighting_rgb: vec![rgb16],
        ..RenderHybridGiReadbackOutputs::default()
    }
}

fn assert_same_extent(lhs: &CapturedFrame, rhs: &CapturedFrame) {
    assert_eq!(lhs.width, rhs.width);
    assert_eq!(lhs.height, rhs.height);
    assert_eq!(lhs.rgba.len(), rhs.rgba.len());
}

fn assert_nonblank(frame: &CapturedFrame, label: &str) {
    let max_channel = frame.rgba.iter().copied().max().unwrap_or_default();

    assert!(
        max_channel > 0,
        "expected {label} render capture to contain visible pixels"
    );
}

fn center_channel_average(frame: &CapturedFrame, channel: usize) -> f32 {
    average_region_channel(
        &frame.rgba,
        frame.width,
        frame.height,
        channel,
        0.25,
        0.75,
        0.25,
        0.75,
    )
}

fn average_luma(frame: &CapturedFrame) -> f32 {
    let mut total = 0.0f32;
    let mut count = 0.0f32;

    for pixel in frame.rgba.chunks_exact(4) {
        total += pixel[0] as f32 * 0.2126 + pixel[1] as f32 * 0.7152 + pixel[2] as f32 * 0.0722;
        count += 1.0;
    }

    if count <= 0.0 {
        0.0
    } else {
        total / count
    }
}

fn write_three_panel_png(path: &Path, frames: [&CapturedFrame; 3]) {
    let panel_width = frames[0].width;
    let panel_height = frames[0].height;
    let output_width = panel_width * 3 + PANEL_SEPARATOR_PX * 2;
    let output_height = panel_height;

    let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(output_width, output_height, |x, y| {
        if x >= panel_width && x < panel_width + PANEL_SEPARATOR_PX {
            return Rgba([24, 28, 36, 255]);
        }
        if x >= panel_width * 2 + PANEL_SEPARATOR_PX && x < panel_width * 2 + PANEL_SEPARATOR_PX * 2
        {
            return Rgba([24, 28, 36, 255]);
        }

        let (frame_index, source_x) = if x < panel_width {
            (0, x)
        } else if x < panel_width * 2 + PANEL_SEPARATOR_PX {
            (1, x - panel_width - PANEL_SEPARATOR_PX)
        } else {
            (2, x - panel_width * 2 - PANEL_SEPARATOR_PX * 2)
        };
        let source_index = ((y * panel_width + source_x) as usize) * 4;
        let rgba = &frames[frame_index].rgba[source_index..source_index + 4];
        Rgba([rgba[0], rgba[1], rgba[2], rgba[3]])
    });

    image
        .save_with_format(path, ImageFormat::Png)
        .expect("write Hybrid GI visual export PNG");
}

fn render_test_output_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render");
    fs::create_dir_all(&dir).expect("create runtime render test output directory");
    dir
}

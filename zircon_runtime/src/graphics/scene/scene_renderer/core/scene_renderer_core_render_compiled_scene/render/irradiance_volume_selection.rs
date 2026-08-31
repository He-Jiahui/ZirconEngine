use crate::core::framework::render::{IrradianceVolumeData, select_irradiance_volume_for_view};
use crate::graphics::scene::resources::{IrradianceVolumeTextureBinding, ResourceStreamer};
use crate::graphics::types::ViewportRenderFrame;

pub(super) fn select_frame_irradiance_volume(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
) -> Option<(IrradianceVolumeData, IrradianceVolumeTextureBinding)> {
    let camera_layers = frame.extract.view.selected_camera_layers();
    let volumes = &frame.extract.lighting.advanced_lighting.irradiance_volumes;
    let visible_world_positions = collect_irradiance_sample_positions(
        !volumes.is_empty(),
        frame
            .extract
            .geometry
            .meshes
            .iter()
            .filter(|mesh| mesh.common.layer_mask.intersects(camera_layers))
            .map(|mesh| mesh.transform.translation),
    );
    visible_world_positions
        .as_deref()
        .and_then(|positions| select_irradiance_volume_for_view(volumes, camera_layers, positions))
        .cloned()
        .and_then(|volume| {
            streamer
                .irradiance_volume_texture(volume.voxels)
                .map(|texture| (volume, texture))
        })
}

pub(super) fn collect_irradiance_sample_positions<T>(
    has_irradiance_volumes: bool,
    positions: impl Iterator<Item = T>,
) -> Option<Vec<T>> {
    has_irradiance_volumes.then(|| positions.collect())
}

use super::*;
use crate::core::math::{Mat4, Quat, Vec3};
use crate::core::resource::ResourceId;

#[test]
fn render_irrvol_gpu_normal_matrix_handles_rotation_and_nonuniform_scale() {
    let world_from_volume = Mat4::from_scale_rotation_translation(
        Vec3::new(4.0, 2.0, 0.5),
        Quat::from_rotation_y(0.7) * Quat::from_rotation_x(-0.35),
        Vec3::new(3.0, -2.0, 5.0),
    );
    let volume = IrradianceVolumeData {
        volume_id: 1,
        transform: world_from_volume.inverse(),
        voxels: ResourceId::from_stable_label("runtime://irradiance-volume/normal-matrix"),
        intensity: 1.0,
        affects_lightmapped_meshes: false,
        priority: 0,
        layer_mask: Default::default(),
    };
    let params = GpuIrradianceVolumeParams::from_volume(&volume);
    let normal_ws = Vec3::new(0.3, 0.8, -0.2).normalize();
    let actual = Vec3::new(
        params.normal_to_volume[0][0] * normal_ws.x
            + params.normal_to_volume[1][0] * normal_ws.y
            + params.normal_to_volume[2][0] * normal_ws.z,
        params.normal_to_volume[0][1] * normal_ws.x
            + params.normal_to_volume[1][1] * normal_ws.y
            + params.normal_to_volume[2][1] * normal_ws.z,
        params.normal_to_volume[0][2] * normal_ws.x
            + params.normal_to_volume[1][2] * normal_ws.y
            + params.normal_to_volume[2][2] * normal_ws.z,
    )
    .normalize();
    let expected = volume
        .transform
        .inverse()
        .transpose()
        .transform_vector3(normal_ws)
        .normalize();

    assert!((actual - expected).length() <= 1.0e-5);
    assert_eq!(std::mem::size_of::<GpuIrradianceVolumeParams>(), 144);
}

#[test]
fn irradiance_volume_params_append_to_the_frame_upload_batch() {
    let production = include_str!("../resources.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("irradiance volume resource test boundary");

    assert!(production.contains("frame_batch.push("));
    assert!(production.contains("WgpuBufferUpload::new("));
    assert!(!production.contains("queue.write_buffer"));
    assert!(!production.contains(".expect("));
    assert!(!production.contains(".unwrap("));
    assert!(!production.contains("panic!("));
}

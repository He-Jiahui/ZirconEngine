//! 六面相机朝向和存储翻转的坐标约定；与宿主 cubemap 采样方向和 CMFT 布局保持一致。
use serde::{Deserialize, Serialize};
use zircon_runtime::core::framework::render::{
    CubemapFace, ProjectionMode, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{Transform, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 序列化的六面身份；映射到宿主 CubemapFace 时不得改变坐标约定。
pub enum ReflectionProbeCaptureFace {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl ReflectionProbeCaptureFace {
    pub const fn cubemap_face(self) -> CubemapFace {
        match self {
            Self::PositiveX => CubemapFace::PositiveX,
            Self::NegativeX => CubemapFace::NegativeX,
            Self::PositiveY => CubemapFace::PositiveY,
            Self::NegativeY => CubemapFace::NegativeY,
            Self::PositiveZ => CubemapFace::PositiveZ,
            Self::NegativeZ => CubemapFace::NegativeZ,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// GPU 相机朝向到存储布局所需的单轴翻转；不是任意贴图方向变换。
pub enum ReflectionProbeCaptureStorageTransform {
    FlipHorizontal,
    FlipVertical,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// 某个面的相机向量与存储变换成对出现；分离使用会破坏探针采样方向。
pub struct ReflectionProbeCaptureFaceView {
    pub face: ReflectionProbeCaptureFace,
    pub forward: [f32; 3],
    pub up: [f32; 3],
    pub storage_transform: ReflectionProbeCaptureStorageTransform,
}

impl ReflectionProbeCaptureFaceView {
    /// 为选定面建立 90 度正方形 HDR 相机；调用方必须提供有效裁剪平面与同一捕获位置。
    pub fn camera(
        self,
        position: [f32; 3],
        near_plane: f32,
        far_plane: f32,
    ) -> ViewportCameraSnapshot {
        let position = Vec3::from_array(position);
        let forward = Vec3::from_array(self.forward);
        let up = Vec3::from_array(self.up);
        ViewportCameraSnapshot {
            transform: Transform::looking_at(position, position + forward, up),
            projection_mode: ProjectionMode::Perspective,
            fov_y_radians: std::f32::consts::FRAC_PI_2,
            z_near: near_plane,
            z_far: far_plane,
            aspect_ratio: 1.0,
            hdr: true,
            ..ViewportCameraSnapshot::default()
        }
    }

    /// 对完整方形 texel 面就地应用该面约定的翻转；face_size 与切片长度不符会断言失败。
    pub fn transform_to_cmft_layout(self, face_size: u32, rendered_texels: &mut [[f32; 4]]) {
        let face_size = face_size as usize;
        assert_eq!(rendered_texels.len(), face_size * face_size);
        match self.storage_transform {
            ReflectionProbeCaptureStorageTransform::FlipHorizontal => {
                for row in rendered_texels.chunks_exact_mut(face_size) {
                    row.reverse();
                }
            }
            ReflectionProbeCaptureStorageTransform::FlipVertical => {
                for y in 0..face_size / 2 {
                    let opposite = face_size - 1 - y;
                    for x in 0..face_size {
                        rendered_texels.swap(y * face_size + x, opposite * face_size + x);
                    }
                }
            }
        }
    }
}

/// 六面共用的轴与翻转表，供捕获生成和坐标一致性测试使用。
pub const REFLECTION_PROBE_CAPTURE_FACE_VIEWS: [ReflectionProbeCaptureFaceView; 6] = [
    ReflectionProbeCaptureFaceView {
        face: ReflectionProbeCaptureFace::PositiveX,
        forward: [1.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        storage_transform: ReflectionProbeCaptureStorageTransform::FlipHorizontal,
    },
    ReflectionProbeCaptureFaceView {
        face: ReflectionProbeCaptureFace::NegativeX,
        forward: [-1.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        storage_transform: ReflectionProbeCaptureStorageTransform::FlipHorizontal,
    },
    ReflectionProbeCaptureFaceView {
        face: ReflectionProbeCaptureFace::PositiveY,
        forward: [0.0, 1.0, 0.0],
        up: [0.0, 0.0, 1.0],
        storage_transform: ReflectionProbeCaptureStorageTransform::FlipVertical,
    },
    ReflectionProbeCaptureFaceView {
        face: ReflectionProbeCaptureFace::NegativeY,
        forward: [0.0, -1.0, 0.0],
        up: [0.0, 0.0, -1.0],
        storage_transform: ReflectionProbeCaptureStorageTransform::FlipVertical,
    },
    ReflectionProbeCaptureFaceView {
        face: ReflectionProbeCaptureFace::PositiveZ,
        forward: [0.0, 0.0, 1.0],
        up: [0.0, 1.0, 0.0],
        storage_transform: ReflectionProbeCaptureStorageTransform::FlipHorizontal,
    },
    ReflectionProbeCaptureFaceView {
        face: ReflectionProbeCaptureFace::NegativeZ,
        forward: [0.0, 0.0, -1.0],
        up: [0.0, 1.0, 0.0],
        storage_transform: ReflectionProbeCaptureStorageTransform::FlipHorizontal,
    },
];

#[cfg(test)]
#[path = "tests/face_view.rs"]
mod tests;

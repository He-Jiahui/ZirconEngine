use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMeshBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub center: [f32; 3],
    pub radius: f32,
}

impl RenderMeshBounds {
    pub fn from_positions(positions: impl IntoIterator<Item = [f32; 3]>) -> Self {
        let mut iter = positions.into_iter();
        let Some(first) = iter.next() else {
            return Self::default();
        };

        let mut min = first;
        let mut max = first;
        for position in iter {
            for axis in 0..3 {
                min[axis] = min[axis].min(position[axis]);
                max[axis] = max[axis].max(position[axis]);
            }
        }

        let center = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        let radius = max_distance_from_center([min, max], center);
        Self {
            min,
            max,
            center,
            radius,
        }
    }
}

fn max_distance_from_center(points: [[f32; 3]; 2], center: [f32; 3]) -> f32 {
    points
        .into_iter()
        .map(|point| {
            let dx = point[0] - center[0];
            let dy = point[1] - center[1];
            let dz = point[2] - center[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .fold(0.0, f32::max)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextClipTransform {
    rows: [[f32; 4]; 4],
}

impl ScreenSpaceUiTextClipTransform {
    pub(in crate::graphics::scene::scene_renderer::ui) fn from_rows(rows: [[f32; 4]; 4]) -> Self {
        let rows = rows
            .iter()
            .flatten()
            .all(|component| component.is_finite())
            .then_some(rows)
            .unwrap_or_else(identity_rows);
        Self { rows }
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn transform_clip_position(
        self,
        position: [f32; 4],
    ) -> [f32; 4] {
        self.rows.map(|row| {
            row.iter()
                .zip(position)
                .map(|(coefficient, component)| coefficient * component)
                .sum()
        })
    }
}

fn identity_rows() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

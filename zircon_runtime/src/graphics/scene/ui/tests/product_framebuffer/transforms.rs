use crate::graphics::scene::scene_renderer::ui::render::text_projection::ScreenSpaceUiTextClipTransform;

pub(super) fn rotation_about_clip_center(
    center: [f32; 2],
    screen_radians: f32,
    viewport: [f32; 2],
) -> ScreenSpaceUiTextClipTransform {
    let cosine = screen_radians.cos();
    let sine = screen_radians.sin();
    let width = viewport[0].max(1.0);
    let height = viewport[1].max(1.0);
    let x_from_y = sine * height / width;
    let y_from_x = -sine * width / height;
    let translate_x = center[0] - cosine * center[0] - x_from_y * center[1];
    let translate_y = center[1] - y_from_x * center[0] - cosine * center[1];
    ScreenSpaceUiTextClipTransform::from_rows([
        [cosine, x_from_y, 0.0, translate_x],
        [y_from_x, cosine, 0.0, translate_y],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

pub(super) fn perspective_about_clip_center(
    center: [f32; 2],
    perspective_x: f32,
    scale: [f32; 2],
) -> ScreenSpaceUiTextClipTransform {
    let constant_w = 1.0 - perspective_x * center[0];
    ScreenSpaceUiTextClipTransform::from_rows([
        [
            center[0] * perspective_x + scale[0],
            0.0,
            0.0,
            center[0] * constant_w - scale[0] * center[0],
        ],
        [
            center[1] * perspective_x,
            scale[1],
            0.0,
            center[1] * constant_w - scale[1] * center[1],
        ],
        [0.0, 0.0, 1.0, 0.0],
        [perspective_x, 0.0, 0.0, constant_w],
    ])
}

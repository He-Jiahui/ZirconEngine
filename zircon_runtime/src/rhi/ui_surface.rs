use crate::core::framework::render::RenderNativeSurfaceTarget;

use super::RhiError;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiSurfaceRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UiSurfaceRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiSurfaceTextStyle {
    Regular,
    Strong,
    Emphasis,
    StrongEmphasis,
}

impl Default for UiSurfaceTextStyle {
    fn default() -> Self {
        Self::Regular
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiSurfaceImagePayload {
    pub resource_key: String,
    pub width: u32,
    pub height: u32,
    pub upload_bytes: u64,
    pub rgba: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiSurfaceCommandKind {
    Quad {
        color: [u8; 4],
        corner_radius: f32,
    },
    Border {
        color: [u8; 4],
        width: f32,
        corner_radius: f32,
    },
    Text {
        text: String,
        color: [u8; 4],
        font_size: f32,
        line_height: f32,
        style: UiSurfaceTextStyle,
    },
    Image {
        payload: UiSurfaceImagePayload,
    },
    Clip,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiSurfaceCommand {
    pub z_index: i32,
    pub frame: UiSurfaceRect,
    pub clip: Option<UiSurfaceRect>,
    pub kind: UiSurfaceCommandKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiSurfaceDescriptor {
    pub label: Option<&'static str>,
    pub width: u32,
    pub height: u32,
    pub target: Option<RenderNativeSurfaceTarget>,
}

impl UiSurfaceDescriptor {
    pub const fn headless(label: &'static str, width: u32, height: u32) -> Self {
        Self {
            label: Some(label),
            width,
            height,
            target: None,
        }
    }

    pub const fn native(
        label: &'static str,
        width: u32,
        height: u32,
        target: RenderNativeSurfaceTarget,
    ) -> Self {
        Self {
            label: Some(label),
            width,
            height,
            target: Some(target),
        }
    }

    pub fn validate(&self) -> Result<(), RhiError> {
        if self.width == 0 || self.height == 0 {
            return Err(RhiError::InvalidSurfaceDescriptor {
                label: self.label.map(str::to_string),
                reason: "width and height must be greater than zero".to_string(),
            });
        }
        Ok(())
    }

    pub fn clamped_size(&self) -> (u32, u32) {
        (self.width.max(1), self.height.max(1))
    }

    #[cfg(feature = "platform-winit")]
    pub fn from_winit_window(
        label: &'static str,
        window: &dyn winit::window::Window,
    ) -> Result<Self, RhiError> {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

        let size = window.surface_size();
        let raw = window
            .window_handle()
            .map_err(|error| RhiError::SurfaceUnavailable(error.to_string()))?
            .as_raw();
        match raw {
            RawWindowHandle::Win32(handle) => Ok(Self::native(
                label,
                size.width.max(1),
                size.height.max(1),
                RenderNativeSurfaceTarget::Win32 {
                    hwnd: handle.hwnd.get() as u64,
                    hinstance: handle.hinstance.map(|hinstance| hinstance.get() as u64),
                },
            )),
            other => Err(RhiError::SurfaceUnavailable(format!(
                "unsupported native window handle for retained UI surface: {other:?}"
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiSurfaceDrawList {
    pub surface_size: (u32, u32),
    pub damage: Option<UiSurfaceRect>,
    pub commands: Vec<UiSurfaceCommand>,
}

impl UiSurfaceDrawList {
    pub fn new(
        surface_size: (u32, u32),
        damage: Option<UiSurfaceRect>,
        commands: Vec<UiSurfaceCommand>,
    ) -> Self {
        Self {
            surface_size: (surface_size.0.max(1), surface_size.1.max(1)),
            damage,
            commands,
        }
    }

    pub fn stats(&self) -> UiSurfacePresentStats {
        let mut stats = UiSurfacePresentStats {
            surface_size: self.surface_size,
            ..UiSurfacePresentStats::default()
        };
        for command in &self.commands {
            if command_effective_rect(command, self).is_none() {
                continue;
            }
            match &command.kind {
                UiSurfaceCommandKind::Quad { .. }
                | UiSurfaceCommandKind::Border { .. }
                | UiSurfaceCommandKind::Text { .. } => {
                    stats.visible_command_count = stats.visible_command_count.saturating_add(1);
                    stats.visible_draw_item_count = stats.visible_draw_item_count.saturating_add(1);
                    stats.draw_calls = stats.draw_calls.saturating_add(1);
                }
                UiSurfaceCommandKind::Image { payload } => {
                    stats.visible_command_count = stats.visible_command_count.saturating_add(1);
                    stats.visible_draw_item_count = stats.visible_draw_item_count.saturating_add(1);
                    stats.draw_calls = stats.draw_calls.saturating_add(1);
                    stats.image_count = stats.image_count.saturating_add(1);
                    if payload.rgba.is_some() {
                        stats.image_upload_bytes = stats
                            .image_upload_bytes
                            .saturating_add(payload.upload_bytes);
                    }
                }
                UiSurfaceCommandKind::Clip => {
                    stats.clip_count = stats.clip_count.saturating_add(1);
                }
            }
        }
        stats
    }
}

fn command_effective_rect(
    command: &UiSurfaceCommand,
    draw_list: &UiSurfaceDrawList,
) -> Option<UiSurfaceRect> {
    let surface = UiSurfaceRect::new(
        0.0,
        0.0,
        draw_list.surface_size.0 as f32,
        draw_list.surface_size.1 as f32,
    );
    let mut rect = rect_intersection(command.frame, surface)?;
    if let Some(clip) = command.clip {
        rect = rect_intersection(rect, clip)?;
    }
    if let Some(damage) = draw_list.damage {
        rect = rect_intersection(rect, damage)?;
    }
    Some(rect)
}

fn rect_intersection(left: UiSurfaceRect, right: UiSurfaceRect) -> Option<UiSurfaceRect> {
    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = (left.x + left.width).min(right.x + right.width);
    let y1 = (left.y + left.height).min(right.y + right.height);
    (x1 > x0 && y1 > y0).then(|| UiSurfaceRect::new(x0, y0, x1 - x0, y1 - y0))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiSurfacePresentStats {
    pub surface_size: (u32, u32),
    pub draw_calls: u64,
    pub visible_command_count: u64,
    pub visible_draw_item_count: u64,
    pub batch_layer_count: u64,
    pub batch_dependency_count: u64,
    pub image_upload_bytes: u64,
    pub image_count: u64,
    pub clip_count: u64,
    pub presented_frame_count: u64,
}

pub trait UiSurfacePresenter: Send {
    fn resize(&mut self, width: u32, height: u32) -> Result<(), RhiError>;
    fn present(&mut self, draw_list: &UiSurfaceDrawList)
        -> Result<UiSurfacePresentStats, RhiError>;
    fn last_present_stats(&self) -> UiSurfacePresentStats;
}

impl<T: UiSurfacePresenter + ?Sized> UiSurfacePresenter for Box<T> {
    fn resize(&mut self, width: u32, height: u32) -> Result<(), RhiError> {
        self.as_mut().resize(width, height)
    }

    fn present(
        &mut self,
        draw_list: &UiSurfaceDrawList,
    ) -> Result<UiSurfacePresentStats, RhiError> {
        self.as_mut().present(draw_list)
    }

    fn last_present_stats(&self) -> UiSurfacePresentStats {
        self.as_ref().last_present_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_list_stats_count_draw_upload_and_clip_commands() {
        let draw_list = UiSurfaceDrawList::new(
            (64, 32),
            Some(UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0)),
            vec![
                UiSurfaceCommand {
                    z_index: 0,
                    frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Clip,
                },
                UiSurfaceCommand {
                    z_index: 1,
                    frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: [1, 2, 3, 255],
                        corner_radius: 6.0,
                    },
                },
                UiSurfaceCommand {
                    z_index: 2,
                    frame: UiSurfaceRect::new(1.0, 1.0, 8.0, 8.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Border {
                        color: [4, 5, 6, 255],
                        width: 1.0,
                        corner_radius: 6.0,
                    },
                },
                UiSurfaceCommand {
                    z_index: 3,
                    frame: UiSurfaceRect::new(0.0, 0.0, 2.0, 2.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Image {
                        payload: UiSurfaceImagePayload {
                            resource_key: "viewport".to_string(),
                            width: 2,
                            height: 2,
                            upload_bytes: 16,
                            rgba: Some(vec![255; 16]),
                        },
                    },
                },
            ],
        );

        let stats = draw_list.stats();
        assert_eq!(stats.surface_size, (64, 32));
        assert_eq!(stats.draw_calls, 3);
        assert_eq!(stats.visible_command_count, 3);
        assert_eq!(stats.visible_draw_item_count, 3);
        assert_eq!(stats.image_count, 1);
        assert_eq!(stats.image_upload_bytes, 16);
        assert_eq!(stats.clip_count, 1);
    }

    #[test]
    fn draw_list_stats_skip_commands_outside_damage() {
        let draw_list = UiSurfaceDrawList::new(
            (64, 32),
            Some(UiSurfaceRect::new(40.0, 20.0, 8.0, 8.0)),
            vec![
                UiSurfaceCommand {
                    z_index: 0,
                    frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: [1, 2, 3, 255],
                        corner_radius: 0.0,
                    },
                },
                UiSurfaceCommand {
                    z_index: 1,
                    frame: UiSurfaceRect::new(42.0, 22.0, 2.0, 2.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Image {
                        payload: UiSurfaceImagePayload {
                            resource_key: "viewport".to_string(),
                            width: 2,
                            height: 2,
                            upload_bytes: 16,
                            rgba: Some(vec![255; 16]),
                        },
                    },
                },
            ],
        );

        let stats = draw_list.stats();

        assert_eq!(stats.draw_calls, 1);
        assert_eq!(stats.visible_command_count, 1);
        assert_eq!(stats.visible_draw_item_count, 1);
        assert_eq!(stats.image_count, 1);
        assert_eq!(stats.image_upload_bytes, 16);
    }

    #[test]
    fn draw_list_stats_do_not_count_cached_images_as_uploads() {
        let draw_list = UiSurfaceDrawList::new(
            (64, 32),
            None,
            vec![UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 2.0, 2.0),
                clip: None,
                kind: UiSurfaceCommandKind::Image {
                    payload: UiSurfaceImagePayload {
                        resource_key: "cached".to_string(),
                        width: 2,
                        height: 2,
                        upload_bytes: 16,
                        rgba: None,
                    },
                },
            }],
        );

        let stats = draw_list.stats();

        assert_eq!(stats.draw_calls, 1);
        assert_eq!(stats.visible_command_count, 1);
        assert_eq!(stats.visible_draw_item_count, 1);
        assert_eq!(stats.image_count, 1);
        assert_eq!(stats.image_upload_bytes, 0);
    }

    #[test]
    fn surface_descriptor_rejects_zero_size() {
        assert_eq!(
            UiSurfaceDescriptor::headless("bad", 0, 1)
                .validate()
                .unwrap_err(),
            RhiError::InvalidSurfaceDescriptor {
                label: Some("bad".to_string()),
                reason: "width and height must be greater than zero".to_string(),
            }
        );
    }
}

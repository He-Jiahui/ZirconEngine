use super::data::FrameRect;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum HostRedrawRequest {
    None,
    Full { frame_update: bool },
    Region(FrameRect),
}

impl HostRedrawRequest {
    pub(crate) fn none() -> Self {
        Self::None
    }

    pub(super) fn full_frame() -> Self {
        Self::Full { frame_update: true }
    }

    pub(crate) fn region(frame: FrameRect) -> Self {
        if visible_frame(&frame) {
            Self::Region(frame)
        } else {
            Self::None
        }
    }

    pub(crate) fn request_redraw(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub(crate) fn requires_frame_update(&self) -> bool {
        matches!(self, Self::Full { frame_update: true })
    }

    pub(crate) fn damage_region(&self) -> Option<&FrameRect> {
        match self {
            Self::Region(frame) => Some(frame),
            Self::None | Self::Full { .. } => None,
        }
    }

    pub(crate) fn merge(self, next: Self) -> Self {
        match (self, next) {
            (Self::Full { frame_update }, Self::Full { frame_update: next }) => Self::Full {
                frame_update: frame_update || next,
            },
            (current @ Self::Full { .. }, _) => current,
            (_, Self::Full { frame_update }) => Self::Full { frame_update },
            (Self::Region(current), Self::Region(next)) => {
                Self::Region(union_frame(&current, &next))
            }
            (Self::None, Self::Region(next)) => Self::Region(next),
            (current, Self::None) => current,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePointerDispatchResult {
    redraw: HostRedrawRequest,
}

impl NativePointerDispatchResult {
    pub(super) fn idle() -> Self {
        Self {
            redraw: HostRedrawRequest::none(),
        }
    }

    pub(super) fn full_frame() -> Self {
        Self {
            redraw: HostRedrawRequest::full_frame(),
        }
    }

    pub(super) fn region(frame: FrameRect) -> Self {
        Self {
            redraw: HostRedrawRequest::region(frame),
        }
    }

    pub(crate) fn request_redraw(&self) -> bool {
        self.redraw.request_redraw()
    }

    pub(crate) fn requires_frame_update(&self) -> bool {
        self.redraw.requires_frame_update()
    }

    pub(crate) fn damage_region(&self) -> Option<FrameRect> {
        self.redraw.damage_region().cloned()
    }

    pub(super) fn redraw(self) -> HostRedrawRequest {
        self.redraw
    }
}

fn visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

fn union_frame(left: &FrameRect, right: &FrameRect) -> FrameRect {
    let x0 = left.x.min(right.x);
    let y0 = left.y.min(right.y);
    let x1 = (left.x + left.width).max(right.x + right.width);
    let y1 = (left.y + left.height).max(right.y + right.height);
    FrameRect {
        x: x0,
        y: y0,
        width: (x1 - x0).max(0.0),
        height: (y1 - y0).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redraw_region_merge_unions_damage_without_frame_update() {
        let redraw = HostRedrawRequest::region(FrameRect {
            x: 8.0,
            y: 10.0,
            width: 20.0,
            height: 12.0,
        })
        .merge(HostRedrawRequest::region(FrameRect {
            x: 24.0,
            y: 6.0,
            width: 10.0,
            height: 20.0,
        }));

        assert_eq!(
            redraw.damage_region(),
            Some(&FrameRect {
                x: 8.0,
                y: 6.0,
                width: 26.0,
                height: 20.0,
            })
        );
        assert!(!redraw.requires_frame_update());
    }

    #[test]
    fn redraw_full_merge_overrides_region_and_preserves_frame_update() {
        let redraw = HostRedrawRequest::region(FrameRect {
            x: 8.0,
            y: 10.0,
            width: 20.0,
            height: 12.0,
        })
        .merge(HostRedrawRequest::Full { frame_update: true });

        assert!(redraw.request_redraw());
        assert!(redraw.requires_frame_update());
        assert_eq!(redraw.damage_region(), None);
    }
}

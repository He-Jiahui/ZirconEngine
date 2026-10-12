use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::super::super::super::PanePointerRoute;

pub(in super::super) struct ViewportBodyRoute<'a> {
    pub(in super::super) body: FrameRect,
    pub(in super::super) toolbar_route: Option<PanePointerRoute<'a>>,
}

impl<'a> ViewportBodyRoute<'a> {
    pub(super) fn content(body: FrameRect) -> Self {
        Self {
            body,
            toolbar_route: None,
        }
    }

    pub(super) fn toolbar(body: FrameRect, toolbar_route: PanePointerRoute<'a>) -> Self {
        Self {
            body,
            toolbar_route: Some(toolbar_route),
        }
    }
}

use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::super::super::super::PanePointerRoute;

pub(in super::super) struct ViewportBodyRoute {
    pub(in super::super) body: FrameRect,
    pub(in super::super) toolbar_route: Option<PanePointerRoute>,
}

impl ViewportBodyRoute {
    pub(super) fn content(body: FrameRect) -> Self {
        Self {
            body,
            toolbar_route: None,
        }
    }

    pub(super) fn toolbar(body: FrameRect, toolbar_route: PanePointerRoute) -> Self {
        Self {
            body,
            toolbar_route: Some(toolbar_route),
        }
    }
}

use zircon_ui::{
    UiAxis, UiContainerKind, UiFrame, UiInputPolicy, UiNodeId, UiNodePath, UiPoint,
    UiPointerDispatcher, UiPointerEvent, UiPointerEventKind, UiScrollState, UiScrollableBoxConfig,
    UiScrollbarVisibility, UiSize, UiStateFlags, UiSurface, UiTreeId, UiTreeNode,
};

use crate::workbench::snapshot::AssetSelectionSnapshot;

const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
const VIEWPORT_NODE_ID: UiNodeId = UiNodeId::new(2);

const ASSET_DETAILS_HEADER_HEIGHT: f32 = 48.0;
const ASSET_DETAILS_VIEWPORT_Y: f32 = ASSET_DETAILS_HEADER_HEIGHT + 1.0;
const ASSET_DETAILS_PADDING: f32 = 12.0;
const ASSET_DETAILS_SECTION_GAP: f32 = 10.0;
const ASSET_DETAILS_PREVIEW_HEIGHT: f32 = 180.0;
const ASSET_DETAILS_LOCATOR_HEIGHT: f32 = 54.0;
const ASSET_DETAILS_TYPE_HEIGHT: f32 = 50.0;
const ASSET_DETAILS_IDENTITY_HEIGHT: f32 = 58.0;
const ASSET_DETAILS_METADATA_HEIGHT: f32 = 64.0;
const ASSET_DETAILS_DIAGNOSTICS_HEIGHT: f32 = 82.0;

const CONSOLE_VIEWPORT_Y: f32 = 0.0;
const CONSOLE_PADDING_TOP: f32 = 8.0;
const CONSOLE_PADDING_BOTTOM: f32 = 12.0;
const CONSOLE_TEXT_WIDTH_INSET: f32 = 20.0;
const CONSOLE_BLOCK_GAP: f32 = 6.0;
const CONSOLE_MONO_LINE_HEIGHT: f32 = 16.0;
const CONSOLE_BODY_LINE_HEIGHT: f32 = 14.0;
const CONSOLE_MONO_CHAR_WIDTH: f32 = 6.6;
const CONSOLE_BODY_CHAR_WIDTH: f32 = 5.8;
const CONSOLE_EMPTY_STATUS: &str = "> No output yet";

const INSPECTOR_VIEWPORT_Y: f32 = 0.0;
const INSPECTOR_VIEWPORT_EXTRA: f32 = 16.0;
const INSPECTOR_ROW_GAP: f32 = 4.0;
const INSPECTOR_HEADER_HEIGHT: f32 = 22.0;
const INSPECTOR_FIELD_HEIGHT: f32 = 22.0;
const INSPECTOR_POSITION_HEIGHT: f32 = 22.0;
const INSPECTOR_DIVIDER_HEIGHT: f32 = 1.0;
const INSPECTOR_ACTIONS_HEIGHT: f32 = 24.0;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScrollSurfacePointerLayout {
    pub pane_size: UiSize,
    pub viewport_origin_y: f32,
    pub content_extent: f32,
}

impl Default for ScrollSurfacePointerLayout {
    fn default() -> Self {
        Self {
            pane_size: UiSize::new(0.0, 0.0),
            viewport_origin_y: 0.0,
            content_extent: 0.0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ScrollSurfacePointerState {
    pub scroll_offset: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScrollSurfacePointerRoute {
    Viewport,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScrollSurfacePointerDispatch {
    pub route: Option<ScrollSurfacePointerRoute>,
    pub state: ScrollSurfacePointerState,
}

pub(crate) struct ScrollSurfacePointerBridge {
    tree_id: &'static str,
    path_prefix: &'static str,
    layout: ScrollSurfacePointerLayout,
    state: ScrollSurfacePointerState,
    surface: UiSurface,
    dispatcher: UiPointerDispatcher,
}

impl ScrollSurfacePointerBridge {
    pub(crate) fn new(tree_id: &'static str, path_prefix: &'static str) -> Self {
        let mut bridge = Self {
            tree_id,
            path_prefix,
            layout: ScrollSurfacePointerLayout::default(),
            state: ScrollSurfacePointerState::default(),
            surface: UiSurface::new(UiTreeId::new(tree_id)),
            dispatcher: UiPointerDispatcher::default(),
        };
        bridge.rebuild_surface();
        bridge
    }

    pub(crate) fn sync(
        &mut self,
        layout: ScrollSurfacePointerLayout,
        state: ScrollSurfacePointerState,
    ) {
        self.layout = layout;
        self.state = state;
        self.clamp_scroll_offset();
        self.rebuild_surface();
    }

    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> Result<ScrollSurfacePointerDispatch, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(
                &self.dispatcher,
                UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta),
            )
            .map_err(|error| error.to_string())?;
        if let Some(viewport) = self.surface.tree.node(VIEWPORT_NODE_ID) {
            let offset = viewport.scroll_state.unwrap_or_default().offset;
            if (self.state.scroll_offset - offset).abs() > f32::EPSILON {
                self.state.scroll_offset = offset;
                self.rebuild_surface();
            }
        }

        Ok(ScrollSurfacePointerDispatch {
            route: map_route(dispatch.handled_by.or(dispatch.route.target)),
            state: self.state.clone(),
        })
    }

    fn clamp_scroll_offset(&mut self) {
        let max_offset =
            (self.layout.content_extent - viewport_frame(&self.layout).height).max(0.0);
        self.state.scroll_offset = self.state.scroll_offset.clamp(0.0, max_offset);
    }

    fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new(self.tree_id));
        surface.tree.insert_root(
            UiTreeNode::new(
                ROOT_NODE_ID,
                UiNodePath::new(format!("{}/root", self.path_prefix)),
            )
            .with_frame(UiFrame::new(
                0.0,
                0.0,
                self.layout.pane_size.width.max(0.0),
                self.layout.pane_size.height.max(0.0),
            ))
            .with_state_flags(base_state(false)),
        );

        let viewport = viewport_frame(&self.layout);
        surface
            .tree
            .insert_child(
                ROOT_NODE_ID,
                UiTreeNode::new(
                    VIEWPORT_NODE_ID,
                    UiNodePath::new(format!("{}/viewport", self.path_prefix)),
                )
                .with_frame(viewport)
                .with_z_index(10)
                .with_input_policy(UiInputPolicy::Receive)
                .with_clip_to_bounds(true)
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: None,
                }))
                .with_scroll_state(UiScrollState {
                    offset: self.state.scroll_offset,
                    viewport_extent: viewport.height.max(0.0),
                    content_extent: self.layout.content_extent.max(0.0),
                })
                .with_state_flags(base_state(true)),
            )
            .expect("scroll surface root must exist");

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = UiPointerDispatcher::default();
    }
}

pub(crate) fn console_scroll_layout(
    pane_size: UiSize,
    content_extent: f32,
) -> ScrollSurfacePointerLayout {
    ScrollSurfacePointerLayout {
        pane_size,
        viewport_origin_y: CONSOLE_VIEWPORT_Y,
        content_extent,
    }
}

pub(crate) fn asset_details_scroll_layout(
    pane_size: UiSize,
    selection: &AssetSelectionSnapshot,
) -> ScrollSurfacePointerLayout {
    ScrollSurfacePointerLayout {
        pane_size,
        viewport_origin_y: ASSET_DETAILS_VIEWPORT_Y,
        content_extent: asset_details_content_extent(selection),
    }
}

pub(crate) fn inspector_scroll_layout(pane_size: UiSize) -> ScrollSurfacePointerLayout {
    ScrollSurfacePointerLayout {
        pane_size,
        viewport_origin_y: INSPECTOR_VIEWPORT_Y,
        content_extent: inspector_content_extent(),
    }
}

pub(crate) fn asset_details_content_extent(selection: &AssetSelectionSnapshot) -> f32 {
    let mut sections = vec![
        ASSET_DETAILS_PREVIEW_HEIGHT,
        ASSET_DETAILS_LOCATOR_HEIGHT,
        ASSET_DETAILS_TYPE_HEIGHT,
        ASSET_DETAILS_IDENTITY_HEIGHT,
        ASSET_DETAILS_METADATA_HEIGHT,
    ];
    if !selection.diagnostics.is_empty() {
        sections.push(ASSET_DETAILS_DIAGNOSTICS_HEIGHT);
    }
    let content = sections.into_iter().sum::<f32>();
    let gaps = (sections_len(selection).saturating_sub(1) as f32) * ASSET_DETAILS_SECTION_GAP;
    ASSET_DETAILS_PADDING * 2.0 + content + gaps
}

pub(crate) fn inspector_content_extent() -> f32 {
    let sections = [
        INSPECTOR_HEADER_HEIGHT,
        INSPECTOR_FIELD_HEIGHT,
        INSPECTOR_FIELD_HEIGHT,
        INSPECTOR_POSITION_HEIGHT,
        INSPECTOR_DIVIDER_HEIGHT,
        INSPECTOR_ACTIONS_HEIGHT,
    ];
    let content = sections.into_iter().sum::<f32>();
    let gaps = (sections.len().saturating_sub(1) as f32) * INSPECTOR_ROW_GAP;
    content + gaps + INSPECTOR_VIEWPORT_EXTRA
}

pub(crate) fn console_content_extent(
    status_text: &str,
    pane_width: f32,
    show_empty: bool,
    empty_body: &str,
) -> f32 {
    let content_width = (pane_width - CONSOLE_TEXT_WIDTH_INSET).max(CONSOLE_BODY_CHAR_WIDTH);
    let status_block = if show_empty {
        if status_text.trim().is_empty() {
            CONSOLE_EMPTY_STATUS.to_string()
        } else {
            format!("> {status_text}")
        }
    } else {
        status_text.to_string()
    };
    let mut height = CONSOLE_PADDING_TOP
        + estimate_text_block_height(
            &status_block,
            content_width,
            CONSOLE_MONO_CHAR_WIDTH,
            CONSOLE_MONO_LINE_HEIGHT,
        )
        + CONSOLE_PADDING_BOTTOM;
    if show_empty && !empty_body.trim().is_empty() {
        height += CONSOLE_BLOCK_GAP
            + estimate_text_block_height(
                empty_body,
                content_width,
                CONSOLE_BODY_CHAR_WIDTH,
                CONSOLE_BODY_LINE_HEIGHT,
            );
    }
    height
}

fn sections_len(selection: &AssetSelectionSnapshot) -> usize {
    if selection.diagnostics.is_empty() {
        5
    } else {
        6
    }
}

fn estimate_text_block_height(text: &str, width: f32, char_width: f32, line_height: f32) -> f32 {
    estimate_wrapped_line_count(text, width, char_width) as f32 * line_height
}

fn estimate_wrapped_line_count(text: &str, width: f32, char_width: f32) -> usize {
    let columns = (width / char_width).floor().max(1.0) as usize;
    text.split('\n')
        .map(|line| {
            let count = line.chars().count();
            if count == 0 {
                1
            } else {
                ((count + columns - 1) / columns).max(1)
            }
        })
        .sum::<usize>()
        .max(1)
}

fn viewport_frame(layout: &ScrollSurfacePointerLayout) -> UiFrame {
    UiFrame::new(
        0.0,
        layout.viewport_origin_y.max(0.0),
        layout.pane_size.width.max(0.0),
        (layout.pane_size.height - layout.viewport_origin_y).max(0.0),
    )
}

fn map_route(target: Option<UiNodeId>) -> Option<ScrollSurfacePointerRoute> {
    match target {
        Some(VIEWPORT_NODE_ID) => Some(ScrollSurfacePointerRoute::Viewport),
        _ => None,
    }
}

fn base_state(interactive: bool) -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: interactive,
        clickable: interactive,
        hoverable: interactive,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

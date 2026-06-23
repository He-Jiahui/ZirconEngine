use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::autolayout::{PaneConstraintOverride, ShellRegionId, WorkbenchConstraintTokenName};
use super::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowId, DocumentNode,
    MainHostPageLayout, MainPageId, SplitAxis, TabStackLayout, WorkbenchLayout,
};

pub const LAYOUT_PRESET_PERSISTENCE_VERSION: u32 = 1;

const LEFT_DRAWER_WIDTH_TOKEN: &str = "--left-drawer-width";
const RIGHT_DRAWER_WIDTH_TOKEN: &str = "--right-drawer-width";
const BOTTOM_OUTPUT_HEIGHT_TOKEN: &str = "--bottom-output-height";
const MIN_PERSISTED_DRAWER_EXTENT: f32 = 120.0;
const DEFAULT_LAYOUT_USER_ID: &str = "default";

const LEFT_DRAWER_SLOTS: [ActivityDrawerSlot; 2] =
    [ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom];
const RIGHT_DRAWER_SLOTS: [ActivityDrawerSlot; 2] = [
    ActivityDrawerSlot::RightTop,
    ActivityDrawerSlot::RightBottom,
];
const BOTTOM_DRAWER_SLOTS: [ActivityDrawerSlot; 1] = [ActivityDrawerSlot::Bottom];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutPresetName {
    Authoring,
    Review,
    Focus,
    Debug,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutPreset {
    pub name: LayoutPresetName,
    pub drawer_states: Vec<LayoutPresetDrawerState>,
    pub size_overrides: Vec<LayoutPresetSizeOverride>,
    pub center_split: CenterSplitLayout,
}

impl LayoutPreset {
    pub fn builtin_presets() -> Vec<Self> {
        vec![
            Self::authoring(),
            Self::review(),
            Self::focus(),
            Self::debug(),
        ]
    }

    pub fn authoring() -> Self {
        Self {
            name: LayoutPresetName::Authoring,
            drawer_states: drawer_states(ActivityDrawerMode::Pinned),
            size_overrides: default_size_overrides(),
            center_split: CenterSplitLayout::SingleDocument,
        }
    }

    pub fn review() -> Self {
        Self {
            name: LayoutPresetName::Review,
            drawer_states: vec![
                drawer_state(ActivityDrawerSlot::LeftTop, ActivityDrawerMode::Collapsed),
                drawer_state(
                    ActivityDrawerSlot::LeftBottom,
                    ActivityDrawerMode::Collapsed,
                ),
                drawer_state(ActivityDrawerSlot::RightTop, ActivityDrawerMode::Pinned),
                drawer_state(ActivityDrawerSlot::RightBottom, ActivityDrawerMode::Pinned),
                drawer_state(ActivityDrawerSlot::Bottom, ActivityDrawerMode::Pinned),
            ],
            size_overrides: default_size_overrides(),
            center_split: CenterSplitLayout::SingleDocument,
        }
    }

    pub fn focus() -> Self {
        Self {
            name: LayoutPresetName::Focus,
            drawer_states: drawer_states(ActivityDrawerMode::Collapsed),
            size_overrides: Vec::new(),
            center_split: CenterSplitLayout::SingleDocument,
        }
    }

    pub fn debug() -> Self {
        Self {
            name: LayoutPresetName::Debug,
            drawer_states: vec![
                drawer_state(ActivityDrawerSlot::LeftTop, ActivityDrawerMode::Collapsed),
                drawer_state(
                    ActivityDrawerSlot::LeftBottom,
                    ActivityDrawerMode::Collapsed,
                ),
                drawer_state(ActivityDrawerSlot::RightTop, ActivityDrawerMode::Collapsed),
                drawer_state(
                    ActivityDrawerSlot::RightBottom,
                    ActivityDrawerMode::Collapsed,
                ),
                drawer_state(ActivityDrawerSlot::Bottom, ActivityDrawerMode::Pinned),
            ],
            size_overrides: vec![LayoutPresetSizeOverride::new(
                "--bottom-output-height",
                320.0,
            )],
            center_split: CenterSplitLayout::Split {
                axis: SplitAxis::Horizontal,
                panes: 2,
            },
        }
    }

    pub fn capture_from_layout(
        name: LayoutPresetName,
        layout: &WorkbenchLayout,
        page_id: &MainPageId,
    ) -> Self {
        let drawers = activity_window_drawers_for_page(layout, page_id)
            .unwrap_or_else(|| layout.drawers.clone());
        Self {
            name,
            drawer_states: drawer_states_from_layout(&drawers),
            size_overrides: size_overrides_from_layout(layout, page_id, &drawers),
            center_split: center_split_from_layout(layout, page_id),
        }
    }

    pub fn apply_to_layout(&self, layout: &mut WorkbenchLayout, page_id: &MainPageId) {
        apply_drawer_states(layout, page_id, &self.drawer_states);
        apply_size_overrides(layout, page_id, &self.size_overrides);
        apply_center_split(layout, page_id, self.center_split);
        if &layout.active_main_page == page_id {
            layout.sync_legacy_drawers_from_active_activity_window();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutPresetDrawerState {
    pub slot: ActivityDrawerSlot,
    pub mode: ActivityDrawerMode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutPresetSizeOverride {
    pub token: WorkbenchConstraintTokenName,
    pub value: i32,
}

impl LayoutPresetSizeOverride {
    pub fn new(token: impl Into<String>, value: f32) -> Self {
        Self {
            token: WorkbenchConstraintTokenName::new(token),
            value: value.round() as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CenterSplitLayout {
    SingleDocument,
    Split { axis: SplitAxis, panes: u8 },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LayoutUserId(String);

impl LayoutUserId {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        if value.trim().is_empty() {
            Self(DEFAULT_LAYOUT_USER_ID.to_string())
        } else {
            Self(value)
        }
    }

    pub fn default_user() -> Self {
        Self::new(DEFAULT_LAYOUT_USER_ID)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LayoutPresetScope {
    pub user_id: LayoutUserId,
    pub page_id: MainPageId,
}

impl LayoutPresetScope {
    pub fn new(user_id: impl Into<String>, page_id: MainPageId) -> Self {
        Self {
            user_id: LayoutUserId::new(user_id),
            page_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedLayoutPreset {
    pub format_version: u32,
    pub preset: LayoutPreset,
}

impl PersistedLayoutPreset {
    pub fn new(preset: LayoutPreset) -> Self {
        Self {
            format_version: LAYOUT_PRESET_PERSISTENCE_VERSION,
            preset,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LayoutPresetPersistenceStore {
    #[serde(default)]
    entries: Vec<LayoutPresetPersistenceEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutPresetPersistenceEntry {
    pub scope: LayoutPresetScope,
    pub document: PersistedLayoutPreset,
}

impl LayoutPresetPersistenceStore {
    pub fn persist_layout(&mut self, scope: LayoutPresetScope, preset: LayoutPreset) {
        self.insert_persisted(scope, PersistedLayoutPreset::new(preset));
    }

    pub fn insert_persisted(&mut self, scope: LayoutPresetScope, document: PersistedLayoutPreset) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.scope == scope) {
            entry.document = document;
        } else {
            self.entries
                .push(LayoutPresetPersistenceEntry { scope, document });
            self.entries
                .sort_by(|left, right| left.scope.cmp(&right.scope));
        }
    }

    pub fn persist_layout_snapshot(
        &mut self,
        scope: LayoutPresetScope,
        name: LayoutPresetName,
        layout: &WorkbenchLayout,
    ) -> LayoutPreset {
        let preset = LayoutPreset::capture_from_layout(name, layout, &scope.page_id);
        self.persist_layout(scope, preset.clone());
        preset
    }

    pub fn restore_layout(&self, scope: &LayoutPresetScope) -> LayoutPresetRestoreResult {
        let Some(document) = self
            .entries
            .iter()
            .find(|entry| &entry.scope == scope)
            .map(|entry| &entry.document)
        else {
            return LayoutPresetRestoreResult::fallback(LayoutPresetRestoreFallback::Missing);
        };

        if document.format_version != LAYOUT_PRESET_PERSISTENCE_VERSION {
            return LayoutPresetRestoreResult::fallback(
                LayoutPresetRestoreFallback::VersionMismatch {
                    stored_version: document.format_version,
                    expected_version: LAYOUT_PRESET_PERSISTENCE_VERSION,
                },
            );
        }

        LayoutPresetRestoreResult::Restored(document.preset.clone())
    }

    pub fn restore_into_layout(
        &self,
        scope: &LayoutPresetScope,
        layout: &mut WorkbenchLayout,
    ) -> LayoutPresetRestoreResult {
        let restored = self.restore_layout(scope);
        restored.preset().apply_to_layout(layout, &scope.page_id);
        restored
    }

    pub fn entries(&self) -> &[LayoutPresetPersistenceEntry] {
        &self.entries
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayoutPresetRestoreFallback {
    Missing,
    VersionMismatch {
        stored_version: u32,
        expected_version: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayoutPresetRestoreResult {
    Restored(LayoutPreset),
    Fallback {
        reason: LayoutPresetRestoreFallback,
        preset: LayoutPreset,
    },
}

impl LayoutPresetRestoreResult {
    fn fallback(reason: LayoutPresetRestoreFallback) -> Self {
        Self::Fallback {
            reason,
            preset: LayoutPreset::authoring(),
        }
    }

    pub fn preset(&self) -> &LayoutPreset {
        match self {
            Self::Restored(preset) | Self::Fallback { preset, .. } => preset,
        }
    }

    pub fn into_preset(self) -> LayoutPreset {
        match self {
            Self::Restored(preset) | Self::Fallback { preset, .. } => preset,
        }
    }

    pub fn fallback_reason(&self) -> Option<&LayoutPresetRestoreFallback> {
        match self {
            Self::Restored(_) => None,
            Self::Fallback { reason, .. } => Some(reason),
        }
    }
}

fn drawer_states(mode: ActivityDrawerMode) -> Vec<LayoutPresetDrawerState> {
    ActivityDrawerSlot::ALL
        .into_iter()
        .map(|slot| drawer_state(slot, mode))
        .collect()
}

fn drawer_state(slot: ActivityDrawerSlot, mode: ActivityDrawerMode) -> LayoutPresetDrawerState {
    LayoutPresetDrawerState { slot, mode }
}

fn default_size_overrides() -> Vec<LayoutPresetSizeOverride> {
    vec![
        LayoutPresetSizeOverride::new(LEFT_DRAWER_WIDTH_TOKEN, 332.0),
        LayoutPresetSizeOverride::new(RIGHT_DRAWER_WIDTH_TOKEN, 404.0),
        LayoutPresetSizeOverride::new(BOTTOM_OUTPUT_HEIGHT_TOKEN, 228.0),
    ]
}

fn drawer_states_from_layout(
    drawers: &BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
) -> Vec<LayoutPresetDrawerState> {
    ActivityDrawerSlot::ALL
        .into_iter()
        .map(|slot| {
            let mode = drawers
                .get(&slot)
                .map(|drawer| drawer.mode)
                .unwrap_or(ActivityDrawerMode::Pinned);
            drawer_state(slot, mode)
        })
        .collect()
}

fn size_overrides_from_layout(
    layout: &WorkbenchLayout,
    page_id: &MainPageId,
    drawers: &BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
) -> Vec<LayoutPresetSizeOverride> {
    let mut overrides = Vec::new();
    push_size_override(
        &mut overrides,
        LEFT_DRAWER_WIDTH_TOKEN,
        preferred_region_extent(layout, page_id, ShellRegionId::Left)
            .or_else(|| drawer_extent(drawers, &LEFT_DRAWER_SLOTS)),
    );
    push_size_override(
        &mut overrides,
        RIGHT_DRAWER_WIDTH_TOKEN,
        preferred_region_extent(layout, page_id, ShellRegionId::Right)
            .or_else(|| drawer_extent(drawers, &RIGHT_DRAWER_SLOTS)),
    );
    push_size_override(
        &mut overrides,
        BOTTOM_OUTPUT_HEIGHT_TOKEN,
        preferred_region_extent(layout, page_id, ShellRegionId::Bottom)
            .or_else(|| drawer_extent(drawers, &BOTTOM_DRAWER_SLOTS)),
    );
    overrides
}

fn push_size_override(
    overrides: &mut Vec<LayoutPresetSizeOverride>,
    token: &str,
    value: Option<f32>,
) {
    if let Some(value) = value {
        overrides.push(LayoutPresetSizeOverride::new(token, value));
    }
}

fn drawer_extent(
    drawers: &BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    slots: &[ActivityDrawerSlot],
) -> Option<f32> {
    slots
        .iter()
        .filter_map(|slot| drawers.get(slot).map(|drawer| drawer.extent))
        .reduce(f32::max)
}

fn preferred_region_extent(
    layout: &WorkbenchLayout,
    page_id: &MainPageId,
    region: ShellRegionId,
) -> Option<f32> {
    activity_window_id_for_page(layout, page_id)
        .and_then(|window_id| layout.activity_windows.get(&window_id))
        .and_then(|window| preferred_from_override(window.region_overrides.get(&region), region))
        .or_else(|| preferred_from_override(layout.region_overrides.get(&region), region))
}

fn preferred_from_override(
    value: Option<&PaneConstraintOverride>,
    region: ShellRegionId,
) -> Option<f32> {
    let override_value = value?;
    match region {
        ShellRegionId::Left | ShellRegionId::Right => override_value.width.preferred,
        ShellRegionId::Bottom => override_value.height.preferred,
        ShellRegionId::Document => None,
    }
}

fn center_split_from_layout(layout: &WorkbenchLayout, page_id: &MainPageId) -> CenterSplitLayout {
    layout
        .main_pages
        .iter()
        .find(|page| page.id() == page_id)
        .and_then(|page| match page {
            MainHostPageLayout::WorkbenchPage {
                document_workspace, ..
            } => Some(document_workspace),
            MainHostPageLayout::ExclusiveActivityWindowPage { .. } => None,
        })
        .map(center_split_from_document)
        .unwrap_or(CenterSplitLayout::SingleDocument)
}

fn center_split_from_document(node: &DocumentNode) -> CenterSplitLayout {
    match node {
        DocumentNode::Tabs(_) => CenterSplitLayout::SingleDocument,
        DocumentNode::SplitNode { axis, .. } => CenterSplitLayout::Split {
            axis: *axis,
            panes: document_leaf_count(node),
        },
    }
}

fn document_leaf_count(node: &DocumentNode) -> u8 {
    match node {
        DocumentNode::Tabs(_) => 1,
        DocumentNode::SplitNode { first, second, .. } => document_leaf_count(first)
            .saturating_add(document_leaf_count(second))
            .max(2),
    }
}

fn activity_window_drawers_for_page(
    layout: &WorkbenchLayout,
    page_id: &MainPageId,
) -> Option<BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>> {
    let window_id = activity_window_id_for_page(layout, page_id)?;
    layout
        .activity_windows
        .get(&window_id)
        .map(|window| window.activity_drawers.clone())
}

fn activity_window_id_for_page(
    layout: &WorkbenchLayout,
    page_id: &MainPageId,
) -> Option<ActivityWindowId> {
    layout
        .main_pages
        .iter()
        .find(|page| page.id() == page_id)
        .and_then(|page| page.activity_window_id().cloned())
}

fn apply_drawer_states(
    layout: &mut WorkbenchLayout,
    page_id: &MainPageId,
    states: &[LayoutPresetDrawerState],
) {
    if let Some(window_id) = activity_window_id_for_page(layout, page_id) {
        if let Some(window) = layout.activity_windows.get_mut(&window_id) {
            for state in states {
                if let Some(drawer) = window.activity_drawers.get_mut(&state.slot.canonical()) {
                    apply_drawer_mode(drawer, state.mode);
                }
            }
        }
    }

    if &layout.active_main_page == page_id {
        for state in states {
            if let Some(drawer) = layout.drawers.get_mut(&state.slot.canonical()) {
                apply_drawer_mode(drawer, state.mode);
            }
        }
    }
}

fn apply_drawer_mode(drawer: &mut ActivityDrawerLayout, mode: ActivityDrawerMode) {
    drawer.mode = mode;
    if mode == ActivityDrawerMode::Collapsed {
        drawer.tab_stack.active_tab = None;
        drawer.active_view = None;
    }
}

fn apply_size_overrides(
    layout: &mut WorkbenchLayout,
    page_id: &MainPageId,
    overrides: &[LayoutPresetSizeOverride],
) {
    let activity_window_id = activity_window_id_for_page(layout, page_id);
    for override_value in overrides {
        let Some(binding) = token_binding(override_value.token.as_str()) else {
            continue;
        };
        let extent = (override_value.value as f32).max(MIN_PERSISTED_DRAWER_EXTENT);
        if let Some(window_id) = &activity_window_id {
            if let Some(window) = layout.activity_windows.get_mut(window_id) {
                set_region_preferred(&mut window.region_overrides, binding.region, extent);
                for slot in binding.slots {
                    if let Some(drawer) = window.activity_drawers.get_mut(slot) {
                        drawer.extent = extent;
                    }
                }
            }
        }

        if &layout.active_main_page == page_id {
            set_region_preferred(&mut layout.region_overrides, binding.region, extent);
            for slot in binding.slots {
                if let Some(drawer) = layout.drawers.get_mut(slot) {
                    drawer.extent = extent;
                }
            }
        }
    }
}

struct SizeOverrideBinding {
    region: ShellRegionId,
    slots: &'static [ActivityDrawerSlot],
}

fn token_binding(token: &str) -> Option<SizeOverrideBinding> {
    match token {
        LEFT_DRAWER_WIDTH_TOKEN => Some(SizeOverrideBinding {
            region: ShellRegionId::Left,
            slots: &LEFT_DRAWER_SLOTS,
        }),
        RIGHT_DRAWER_WIDTH_TOKEN => Some(SizeOverrideBinding {
            region: ShellRegionId::Right,
            slots: &RIGHT_DRAWER_SLOTS,
        }),
        BOTTOM_OUTPUT_HEIGHT_TOKEN => Some(SizeOverrideBinding {
            region: ShellRegionId::Bottom,
            slots: &BOTTOM_DRAWER_SLOTS,
        }),
        _ => None,
    }
}

fn set_region_preferred(
    overrides: &mut BTreeMap<ShellRegionId, PaneConstraintOverride>,
    region: ShellRegionId,
    value: f32,
) {
    let override_value = overrides.entry(region).or_default();
    match region {
        ShellRegionId::Left | ShellRegionId::Right => {
            override_value.width.preferred = Some(value);
        }
        ShellRegionId::Bottom => {
            override_value.height.preferred = Some(value);
        }
        ShellRegionId::Document => {}
    }
}

fn apply_center_split(
    layout: &mut WorkbenchLayout,
    page_id: &MainPageId,
    center_split: CenterSplitLayout,
) {
    let Some(document_workspace) = layout
        .main_pages
        .iter_mut()
        .find(|page| page.id() == page_id)
        .and_then(MainHostPageLayout::document_workspace_mut)
    else {
        return;
    };

    match center_split {
        CenterSplitLayout::SingleDocument => {
            let collapsed = collapse_document_tabs(document_workspace);
            *document_workspace = DocumentNode::Tabs(collapsed);
        }
        CenterSplitLayout::Split { axis, panes } => {
            let collapsed = DocumentNode::Tabs(collapse_document_tabs(document_workspace));
            *document_workspace = split_document_for_panes(axis, panes.max(2), collapsed);
        }
    }
}

fn collapse_document_tabs(node: &DocumentNode) -> TabStackLayout {
    let mut tabs = Vec::new();
    let mut active_tab = None;
    collect_document_tabs(node, &mut tabs, &mut active_tab);
    if active_tab
        .as_ref()
        .map(|active| !tabs.contains(active))
        .unwrap_or(true)
    {
        active_tab = tabs.first().cloned();
    }
    TabStackLayout { tabs, active_tab }
}

fn collect_document_tabs(
    node: &DocumentNode,
    tabs: &mut Vec<super::view::ViewInstanceId>,
    active_tab: &mut Option<super::view::ViewInstanceId>,
) {
    match node {
        DocumentNode::Tabs(stack) => {
            for tab in &stack.tabs {
                if !tabs.contains(tab) {
                    tabs.push(tab.clone());
                }
            }
            if active_tab.is_none() {
                *active_tab = stack.active_tab.clone();
            }
        }
        DocumentNode::SplitNode { first, second, .. } => {
            collect_document_tabs(first, tabs, active_tab);
            collect_document_tabs(second, tabs, active_tab);
        }
    }
}

fn split_document_for_panes(axis: SplitAxis, panes: u8, first: DocumentNode) -> DocumentNode {
    if panes <= 1 {
        return first;
    }

    DocumentNode::SplitNode {
        axis,
        ratio: 0.5,
        first: Box::new(first),
        second: Box::new(empty_split_tail(axis, panes.saturating_sub(1))),
    }
}

fn empty_split_tail(axis: SplitAxis, panes: u8) -> DocumentNode {
    if panes <= 1 {
        return DocumentNode::default();
    }

    DocumentNode::SplitNode {
        axis,
        ratio: 0.5,
        first: Box::new(DocumentNode::default()),
        second: Box::new(empty_split_tail(axis, panes.saturating_sub(1))),
    }
}

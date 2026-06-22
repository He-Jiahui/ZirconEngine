use super::super::clip_frame::ProjectedClipFrame;
use super::super::collection_projection::ProjectedCollection;
use super::super::drag_overlay::ProjectedDragOverlayData;
use super::super::popup_actions::ProjectedPopupActions;
use super::super::selection_options::ProjectedSelectionOptions;
use super::super::text_layout::ProjectedTextLayout;
use super::super::validation_state::ProjectedValidationState;
use super::super::value_media::ProjectedValueMedia;
use super::super::visual_state::ProjectedVisualState;
use super::super::visual_style::ProjectedVisualStyle;
use super::super::world_space::ProjectedWorldSpace;

pub(in super::super) struct ProjectedTemplateNodeParts {
    pub(in super::super) node_id: String,
    pub(in super::super) control_id: String,
    pub(in super::super) role: String,
    pub(in super::super) component_role: String,
    pub(in super::super) text_layout: ProjectedTextLayout,
    pub(in super::super) value_media: ProjectedValueMedia,
    pub(in super::super) validation_state: ProjectedValidationState,
    pub(in super::super) selection_options: ProjectedSelectionOptions,
    pub(in super::super) collection: ProjectedCollection,
    pub(in super::super) world_space: ProjectedWorldSpace,
    pub(in super::super) popup_actions: ProjectedPopupActions,
    pub(in super::super) drag_overlay: ProjectedDragOverlayData,
    pub(in super::super) visual_state: ProjectedVisualState,
    pub(in super::super) visual_style: ProjectedVisualStyle,
    pub(in super::super) clip_frame: ProjectedClipFrame,
}

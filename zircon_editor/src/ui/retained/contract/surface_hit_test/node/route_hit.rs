use super::popup_rows::normalized_menu_row_action_id;
use super::{TemplateNodePointerHit, TemplateNodePointerMoveKind, TemplateNodePointerRouteHit};

impl<'a> TemplateNodePointerRouteHit<'a> {
    pub(super) fn with_pane(
        mut self,
        pane: &'a crate::ui::retained_host::host_contract::data::PaneData,
    ) -> Self {
        self.pane_id = pane.id.as_str();
        if matches!(
            self.dispatch_kind,
            "ui_asset_palette" | "ui_asset_palette_page" | "ui_asset_palette_search"
        ) {
            self.palette_search_source = pane
                .ui_asset
                .collections
                .palette_search
                .as_ref()
                .zip(pane.body_surface_frame.as_ref());
        }
        self
    }

    pub(crate) fn to_owned_hit(&self) -> TemplateNodePointerHit {
        let action_id = if self.kind == TemplateNodePointerMoveKind::MenuItem {
            normalized_menu_row_action_id(self.action_id, self.value_text)
        } else {
            self.action_id.into()
        };
        TemplateNodePointerHit {
            palette_search_target: self.palette_search_source.map(|(snapshot, frame)| {
                crate::ui::retained_host::host_contract::data::UiAssetPaletteInputTarget::new(
                    self.pane_id.into(),
                    snapshot.clone(),
                    std::sync::Arc::clone(frame),
                )
            }),
            pane_id: self.pane_id.into(),
            control_id: self.control_id.into(),
            action_id,
            binding_id: self.binding_id.into(),
            dispatch_kind: self.dispatch_kind.into(),
            component_role: self.component_role.into(),
            component_family: self.component_family,
            value_text: self.value_text.into(),
            edit_action_id: self.edit_action_id.into(),
            commit_action_id: self.commit_action_id.into(),
            disabled: self.disabled,
            frame: self.frame.clone(),
            table_row_source_index: self.table_row_source_index,
            table_row_identity_kind: self.table_row_identity_kind.into(),
            table_row_identity_text: self.table_row_identity_text.into(),
        }
    }
}

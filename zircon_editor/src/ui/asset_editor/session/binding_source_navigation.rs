use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;

use crate::ui::asset_editor::{
    source::source_buffer::UiAssetSourceSnapshot, UiAssetBindingSourceDeclaration,
    UiAssetBindingSourceError as Error, UiAssetBindingSourceScope as Scope,
    UiAssetBindingSourceSelector,
};

use super::{
    binding_inspector::reconcile_selected_binding_payload_key,
    ui_asset_editor_session::{
        UiAssetEditorSession, UiAssetSourceCursorAnchor, UiAssetSourceSchema,
    },
};

/// An owned local source result, not an external Definition or a dispatch permit.
#[derive(Clone, Debug)]
pub(crate) struct UiAssetBindingSourceLocation {
    declaration: UiAssetBindingSourceDeclaration,
    scope: Scope,
    request_buffer_revision: u64,
    last_valid_generation: u64,
    source_digest: blake3::Hash,
    route_asset_id: String,
    buffer_snapshot: UiAssetSourceSnapshot,
}

impl UiAssetBindingSourceLocation {
    pub(crate) fn declaration(&self) -> &UiAssetBindingSourceDeclaration {
        &self.declaration
    }

    pub(crate) fn scope(&self) -> Scope {
        self.scope
    }

    pub(crate) fn request_buffer_revision(&self) -> u64 {
        self.request_buffer_revision
    }

    pub(crate) fn last_valid_generation(&self) -> u64 {
        self.last_valid_generation
    }

    pub(crate) fn range_source_digest(&self) -> blake3::Hash {
        self.source_digest
    }
}

impl UiAssetEditorSession {
    pub(crate) fn locate_binding_source(
        &self,
        selector: &UiAssetBindingSourceSelector,
        scope: Scope,
    ) -> Result<UiAssetBindingSourceLocation, Error> {
        if self.source_schema != UiAssetSourceSchema::V2 {
            return Err(Error::UnsupportedSchema);
        }
        let document = self
            .last_valid_v2_document
            .as_ref()
            .ok_or(Error::UnsupportedSchema)?;
        let source = match scope {
            Scope::Current => {
                // Preview/compile diagnostics do not invalidate a parsed source declaration.
                if self.source_buffer.text() != self.last_valid_source_text {
                    return Err(Error::InvalidCurrentSource);
                }
                self.source_buffer.text()
            }
            Scope::LastValid => self.last_valid_source_text.as_str(),
        };
        let outline = match scope {
            Scope::Current => self.source_outline_index(),
            Scope::LastValid => self.last_valid_source_outline_index(),
        };
        let declaration = outline
            .binding_sources(source)
            .locate(source, document, selector)?;
        Ok(UiAssetBindingSourceLocation {
            declaration,
            scope,
            request_buffer_revision: self.source_buffer.revision(),
            last_valid_generation: self.last_valid_source_generation,
            source_digest: blake3::hash(source.as_bytes()),
            route_asset_id: self.route.asset_id.clone(),
            buffer_snapshot: self.source_buffer.snapshot(),
        })
    }

    pub(crate) fn select_binding_source_location(
        &mut self,
        location: &UiAssetBindingSourceLocation,
    ) -> Result<bool, Error> {
        if !self
            .source_buffer
            .is_current_snapshot(&location.buffer_snapshot)
            || self.route.asset_id != location.route_asset_id
            || self.last_valid_source_generation != location.last_valid_generation
        {
            return Err(Error::StaleLocation);
        }
        if location.scope == Scope::LastValid {
            return Err(Error::LastValidReadOnly);
        }
        if self.source_schema != UiAssetSourceSchema::V2
            || self.source_buffer.text() != self.last_valid_source_text
        {
            return Err(Error::InvalidCurrentSource);
        }
        let declaration = &location.declaration;
        if self
            .last_valid_document
            .node(&declaration.node_id)
            .and_then(|node| node.bindings.get(declaration.source_binding_index))
            != Some(&declaration.binding)
        {
            return Err(Error::SelectionUnavailable);
        }
        let line_offset = self
            .source_outline_index()
            .entry_for_node(&declaration.node_id)
            .map_or(0, |entry| {
                declaration
                    .declaration
                    .start
                    .line
                    .saturating_sub(entry.line as usize)
            });
        let changed = self.selection.primary_node_id.as_deref()
            != Some(declaration.node_id.as_str())
            || self.source_cursor_byte_offset != declaration.declaration.start.byte_offset
            || self.selected_binding_index != Some(declaration.source_binding_index);
        self.select_node_id(&declaration.node_id);
        self.selected_binding_index = Some(declaration.source_binding_index);
        self.selected_binding_payload_key = reconcile_selected_binding_payload_key(
            &self.last_valid_document,
            &self.selection,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        self.source_cursor_anchor = Some(UiAssetSourceCursorAnchor {
            node_id: declaration.node_id.clone(),
            line_offset,
        });
        // This offset was checked on the same retained source allocation. Never clamp it.
        self.source_cursor_byte_offset = declaration.declaration.start.byte_offset;
        Ok(changed)
    }
}

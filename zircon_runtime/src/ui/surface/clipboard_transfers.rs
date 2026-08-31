use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    dispatch::{
        UiClipboardRequest, UiClipboardRequestKind, UiClipboardTransferId,
        UiClipboardTransferIntent,
    },
    event_ui::UiNodeId,
};

use super::UiSurface;

#[derive(Default, Serialize, Deserialize)]
pub(super) struct UiSurfaceClipboardTransferStore {
    #[serde(skip)]
    revisions: BTreeMap<UiNodeId, Option<u64>>,
    #[serde(skip)]
    pending: BTreeMap<UiClipboardTransferId, UiClipboardPendingTransfer>,
    #[serde(skip)]
    by_owner: BTreeMap<UiNodeId, UiClipboardTransferId>,
}

impl fmt::Debug for UiSurfaceClipboardTransferStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UiSurfaceClipboardTransferStore")
            .field("revision_owner_count", &self.revisions.len())
            .field("pending_count", &self.pending.len())
            .finish()
    }
}

impl Clone for UiSurfaceClipboardTransferStore {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for UiSurfaceClipboardTransferStore {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::surface) struct UiClipboardPendingTransfer {
    pub(in crate::ui::surface) owner: UiNodeId,
    pub(in crate::ui::surface) property: String,
    pub(in crate::ui::surface) intent: UiClipboardTransferIntent,
    pub(in crate::ui::surface) expected_edit_revision: u64,
    pub(in crate::ui::surface) secure: bool,
}

#[derive(Clone)]
pub(crate) struct UiSurfaceClipboardTransferSnapshot {
    revisions: BTreeMap<UiNodeId, Option<u64>>,
    pending: BTreeMap<UiClipboardTransferId, UiClipboardPendingTransfer>,
    by_owner: BTreeMap<UiNodeId, UiClipboardTransferId>,
}

impl UiSurfaceClipboardTransferStore {
    fn issue(
        &mut self,
        owner: UiNodeId,
        property: String,
        intent: UiClipboardTransferIntent,
        text: Option<String>,
        secure: bool,
    ) -> Option<UiClipboardRequest> {
        let expected_edit_revision = self.current_revision(owner)?;
        if let Some(previous) = self.by_owner.remove(&owner) {
            self.pending.remove(&previous);
        }
        let transfer_id = UiClipboardTransferId::issue();
        self.pending.insert(
            transfer_id,
            UiClipboardPendingTransfer {
                owner,
                property,
                intent,
                expected_edit_revision,
                secure,
            },
        );
        self.by_owner.insert(owner, transfer_id);
        let kind = match intent {
            UiClipboardTransferIntent::Copy | UiClipboardTransferIntent::Cut => {
                UiClipboardRequestKind::WriteText
            }
            UiClipboardTransferIntent::Paste => UiClipboardRequestKind::ReadText,
        };
        Some(UiClipboardRequest {
            transfer_id,
            intent,
            expected_edit_revision,
            kind,
            owner,
            text,
        })
    }

    fn take(&mut self, transfer_id: UiClipboardTransferId) -> Option<UiClipboardPendingTransfer> {
        let pending = self.pending.remove(&transfer_id)?;
        if self.by_owner.get(&pending.owner) == Some(&transfer_id) {
            self.by_owner.remove(&pending.owner);
        }
        Some(pending)
    }

    fn cancel(&mut self, transfer_id: UiClipboardTransferId) {
        let _ = self.take(transfer_id);
    }

    fn current_revision(&self, owner: UiNodeId) -> Option<u64> {
        self.revisions.get(&owner).copied().unwrap_or(Some(0))
    }

    fn has_pending_for(&self, owner: UiNodeId) -> bool {
        !self.pending.is_empty() && self.by_owner.contains_key(&owner)
    }

    fn invalidate_owner(&mut self, owner: UiNodeId) {
        let Some(transfer_id) = self.by_owner.get(&owner) else {
            return;
        };
        let Some(pending) = self.pending.get(transfer_id) else {
            return;
        };
        if self.current_revision(owner) != Some(pending.expected_edit_revision) {
            return;
        }
        let next = self
            .current_revision(owner)
            .and_then(|revision| revision.checked_add(1));
        self.revisions.insert(owner, next);
    }

    fn drop_owner(&mut self, owner: UiNodeId) {
        if let Some(transfer_id) = self.by_owner.remove(&owner) {
            self.pending.remove(&transfer_id);
        }
        self.revisions.remove(&owner);
    }

    pub(crate) fn snapshot(&self) -> UiSurfaceClipboardTransferSnapshot {
        UiSurfaceClipboardTransferSnapshot {
            revisions: self.revisions.clone(),
            pending: self.pending.clone(),
            by_owner: self.by_owner.clone(),
        }
    }

    pub(crate) fn restore(&mut self, snapshot: UiSurfaceClipboardTransferSnapshot) {
        self.revisions = snapshot.revisions;
        self.pending = snapshot.pending;
        self.by_owner = snapshot.by_owner;
    }
}

impl UiSurface {
    pub(in crate::ui::surface) fn begin_clipboard_transfer(
        &mut self,
        owner: UiNodeId,
        property: String,
        intent: UiClipboardTransferIntent,
        text: Option<String>,
    ) -> Option<UiClipboardRequest> {
        let secure = super::editable_text_input_is_secure(self, owner);
        self.clipboard_transfers
            .issue(owner, property, intent, text, secure)
    }

    pub(in crate::ui::surface) fn take_clipboard_transfer(
        &mut self,
        transfer_id: UiClipboardTransferId,
    ) -> Option<UiClipboardPendingTransfer> {
        self.clipboard_transfers.take(transfer_id)
    }

    pub(in crate::ui) fn cancel_clipboard_transfer(&mut self, transfer_id: UiClipboardTransferId) {
        self.clipboard_transfers.cancel(transfer_id);
    }

    pub(in crate::ui::surface) fn clipboard_edit_revision(&self, owner: UiNodeId) -> Option<u64> {
        self.clipboard_transfers.current_revision(owner)
    }

    pub(in crate::ui::surface) fn has_pending_clipboard_transfer(&self, owner: UiNodeId) -> bool {
        self.clipboard_transfers.has_pending_for(owner)
    }

    pub(in crate::ui::surface) fn invalidate_clipboard_transfers_for(&mut self, owner: UiNodeId) {
        self.clipboard_transfers.invalidate_owner(owner);
    }

    pub(in crate::ui::surface) fn drop_clipboard_transfers_for(&mut self, owner: UiNodeId) {
        self.clipboard_transfers.drop_owner(owner);
    }
}

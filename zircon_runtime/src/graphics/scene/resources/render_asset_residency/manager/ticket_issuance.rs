use super::{PreparedReconciliation, PreparedReferenceChange, RenderAssetResidencyManager};
use crate::graphics::scene::resources::render_asset_residency::{
    RenderAssetResidencyAdmissionError, RenderAssetResidencyTicketId,
};

impl RenderAssetResidencyManager {
    pub(super) fn issue_reference_change_tickets(
        &mut self,
        prepared: &mut [PreparedReferenceChange],
    ) -> Result<(), RenderAssetResidencyAdmissionError> {
        let request_count = prepared
            .iter()
            .filter(|change| change.request_seed.is_some())
            .count();
        let mut next_ticket_id = self.reserve_ticket_ids(request_count)?;
        for change in prepared {
            if let Some(seed) = change.request_seed {
                let Some(id) = RenderAssetResidencyTicketId::new(next_ticket_id) else {
                    return Err(RenderAssetResidencyAdmissionError::TicketIdExhausted);
                };
                change.request = Some(seed.issue(id));
                next_ticket_id = next_ticket_id.saturating_add(1);
            }
        }
        self.next_ticket_id = next_ticket_id;
        Ok(())
    }

    pub(super) fn issue_reconciliation_tickets(
        &mut self,
        prepared: &mut [PreparedReconciliation],
    ) -> Result<(), RenderAssetResidencyAdmissionError> {
        let request_count = prepared
            .iter()
            .filter(|change| change.request_seed.is_some())
            .count();
        let mut next_ticket_id = self.reserve_ticket_ids(request_count)?;
        for reconciliation in prepared {
            if let Some(seed) = reconciliation.request_seed {
                let Some(id) = RenderAssetResidencyTicketId::new(next_ticket_id) else {
                    return Err(RenderAssetResidencyAdmissionError::TicketIdExhausted);
                };
                reconciliation.request = Some(seed.issue(id));
                next_ticket_id = next_ticket_id.saturating_add(1);
            }
        }
        self.next_ticket_id = next_ticket_id;
        Ok(())
    }

    pub(super) fn reserve_ticket_ids(
        &self,
        request_count: usize,
    ) -> Result<u64, RenderAssetResidencyAdmissionError> {
        let Ok(request_count) = u64::try_from(request_count) else {
            return Err(RenderAssetResidencyAdmissionError::TicketIdExhausted);
        };
        if self.next_ticket_id == 0 || self.next_ticket_id.checked_add(request_count).is_none() {
            return Err(RenderAssetResidencyAdmissionError::TicketIdExhausted);
        }
        Ok(self.next_ticket_id)
    }
}

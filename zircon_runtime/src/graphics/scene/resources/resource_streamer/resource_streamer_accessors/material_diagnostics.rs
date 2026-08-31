use crate::core::framework::render::{
    RenderMaterialIssueState, RenderMaterialManagementIssueIndex,
    RenderMaterialManagementIssueKind, RenderMaterialManagementIssueView,
    RenderMaterialManagementOverview, RenderMaterialManagementQuery,
    RenderMaterialManagementQueryResult, RenderMaterialManagementQuerySelection,
    RenderMaterialManagementRecord, RenderMaterialManagementRecordSet,
    RenderMaterialManagementRecordSummary, RenderMaterialManagementSelection,
    RenderMaterialManagementSnapshot, RenderMaterialManagementSortOrder,
    RenderMaterialManagementStatusIndex, RenderMaterialManagementStatusView,
    RenderMaterialPreparedState, RenderMaterialPropertyUniformField,
    RenderMaterialPropertyUniformSummary, RenderMaterialPropertyUniformUnsupported,
    RenderMaterialPropertyValueState, RenderMaterialPropertyValueSummary,
    RenderMaterialReadinessReport, RenderMaterialReadinessStatus, RenderMaterialTextureSlotState,
    RenderMaterialTextureSlotSummary,
};
use crate::core::resource::ResourceId;

use super::super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn material_uniform_payload_byte_len(&self, id: &ResourceId) -> Option<u64> {
        self.latest_prepared_material_bundle(id)
            .map(|prepared| prepared.uniform.payload_byte_len())
    }

    pub(crate) fn material_uniform_buffer_byte_len(&self, id: &ResourceId) -> Option<u64> {
        self.latest_prepared_material_bundle(id)
            .map(|prepared| prepared.uniform.buffer_byte_len())
    }

    pub(crate) fn material_uniform_field_count(&self, id: &ResourceId) -> Option<usize> {
        self.latest_prepared_material_bundle(id).map(|prepared| {
            prepared
                .runtime
                .shader_property_uniform_payload
                .layout
                .len()
        })
    }

    pub(crate) fn material_uniform_unsupported_count(&self, id: &ResourceId) -> Option<usize> {
        self.latest_prepared_material_bundle(id).map(|prepared| {
            prepared
                .runtime
                .shader_property_uniform_payload
                .unsupported
                .len()
        })
    }

    pub(crate) fn material_uniform_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialPropertyUniformSummary> {
        self.latest_prepared_material_bundle(id)
            .map(|prepared| prepared.runtime.shader_property_uniform_payload.summary())
    }

    pub(crate) fn material_uniform_fields(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialPropertyUniformField>> {
        self.material_readiness_report(id)
            .map(|report| report.uniform_fields.clone())
    }

    pub(crate) fn material_uniform_unsupported(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialPropertyUniformUnsupported>> {
        self.material_readiness_report(id)
            .map(|report| report.uniform_unsupported.clone())
    }

    pub(crate) fn material_property_value_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialPropertyValueSummary> {
        self.latest_prepared_material_bundle(id).map(|prepared| {
            RenderMaterialPropertyValueSummary::from_values(
                &prepared.runtime.shader_property_values,
            )
        })
    }

    pub(crate) fn material_property_value_states(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialPropertyValueState>> {
        self.material_readiness_report(id)
            .map(|report| report.property_value_states.clone())
    }

    pub(crate) fn material_standard_texture_slot_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialTextureSlotSummary> {
        self.material_readiness_report(id)
            .and_then(|report| report.standard_texture_slot_summary)
    }

    pub(crate) fn material_standard_texture_slot_states(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialTextureSlotState>> {
        self.material_readiness_report(id)
            .map(|report| report.standard_texture_slot_states.clone())
    }

    pub(crate) fn material_texture_slot_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialTextureSlotSummary> {
        self.latest_prepared_material_bundle(id).map(|prepared| {
            RenderMaterialTextureSlotSummary::from_non_standard_slots(
                &prepared.runtime.non_standard_texture_slots,
            )
        })
    }

    pub(crate) fn material_texture_slot_states(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialTextureSlotState>> {
        self.material_readiness_report(id)
            .map(|report| report.non_standard_texture_slot_states.clone())
    }

    pub(crate) fn material_readiness_status(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialReadinessStatus> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::status)
    }

    pub(crate) fn material_issue_state(&self, id: &ResourceId) -> Option<RenderMaterialIssueState> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::issue_state)
    }

    pub(crate) fn material_management_snapshot(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialManagementSnapshot> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::management_snapshot)
    }

    pub(crate) fn material_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialManagementRecord> {
        self.material_readiness_report(id)
            .map(|report| report.management_record(*id))
    }

    pub(crate) fn material_management_records(&self) -> Vec<RenderMaterialManagementRecord> {
        let mut records = self
            .materials
            .keys()
            .filter_map(|id| {
                self.material_readiness_report(id)
                    .map(|report| report.management_record(*id))
            })
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.material_id);
        records
    }

    pub(crate) fn material_management_record_set(&self) -> RenderMaterialManagementRecordSet {
        RenderMaterialManagementRecordSet::from_records(self.material_management_records())
    }

    pub(crate) fn material_management_record_set_sorted(
        &self,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementRecordSet {
        RenderMaterialManagementRecordSet::from_sorted_records(
            self.material_management_records(),
            sort_order,
        )
    }

    pub(crate) fn material_management_overview(&self) -> RenderMaterialManagementOverview {
        self.material_management_record_set().overview()
    }

    pub(crate) fn material_management_overview_sorted(
        &self,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementOverview {
        self.material_management_record_set_sorted(sort_order)
            .overview()
    }

    pub(crate) fn material_management_query(
        &self,
        query: RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQueryResult {
        self.material_management_record_set().query(query)
    }

    pub(crate) fn material_management_query_selection(
        &self,
        query: RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQuerySelection {
        self.material_management_record_set().query_selection(query)
    }

    pub(crate) fn material_management_selection(
        &self,
        material_ids: impl IntoIterator<Item = ResourceId>,
    ) -> RenderMaterialManagementSelection {
        self.material_management_record_set().select(material_ids)
    }

    pub(crate) fn material_management_status_index(&self) -> RenderMaterialManagementStatusIndex {
        self.material_management_record_set().status_index
    }

    pub(crate) fn material_management_issue_index(&self) -> RenderMaterialManagementIssueIndex {
        self.material_management_record_set().issue_index
    }

    pub(crate) fn material_management_issue_view(
        &self,
        issue_kind: RenderMaterialManagementIssueKind,
    ) -> RenderMaterialManagementIssueView {
        self.material_management_record_set().issue_view(issue_kind)
    }

    pub(crate) fn material_management_issue_view_sorted(
        &self,
        issue_kind: RenderMaterialManagementIssueKind,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementIssueView {
        self.material_management_record_set_sorted(sort_order)
            .issue_view(issue_kind)
    }

    pub(crate) fn material_management_status_view(
        &self,
        status: RenderMaterialReadinessStatus,
    ) -> RenderMaterialManagementStatusView {
        self.material_management_record_set().status_view(status)
    }

    pub(crate) fn material_management_status_view_sorted(
        &self,
        status: RenderMaterialReadinessStatus,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementStatusView {
        self.material_management_record_set_sorted(sort_order)
            .status_view(status)
    }

    pub(crate) fn material_management_record_summary(
        &self,
    ) -> RenderMaterialManagementRecordSummary {
        self.material_management_record_set().summary
    }

    pub(crate) fn material_prepared_state(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialPreparedState> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::prepared_state)
    }
}

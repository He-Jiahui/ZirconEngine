use std::sync::Arc;

use zircon_runtime::ui::surface::{
    UiPropertyMutationReport, UiPropertyMutationRequest, UiPropertyMutationStatus,
};
use zircon_runtime_interface::ui::{
    binding::{UiBindingDirtyDomain, UiBindingSourceKind, UiBindingUpdateStatus},
    component::UiValue,
    event_ui::{UiNodeId, UiReflectedPropertySource},
    surface::UiSurfaceFrame,
};

use super::super::UiAssetPreviewHost;
use super::{
    UiPreviewVisibilityError as Error, UiPreviewVisibilityGesture,
    UiPreviewVisibilityOutcome as Outcome, UiPreviewVisibilityPublication as Publication,
};

impl UiAssetPreviewHost {
    pub(in crate::ui::asset_editor) fn apply_visibility_gesture(
        &mut self,
        gesture: &UiPreviewVisibilityGesture,
        source_frame: &Arc<UiSurfaceFrame>,
        sequence: u64,
        visible: Option<bool>,
    ) -> Outcome {
        if let Err(error) = self.admit_visibility_request(gesture, source_frame, sequence) {
            return Outcome::Rejected(error);
        }
        let state = self.visibility_state.as_mut().expect("validated gesture");
        state.last_sequence = sequence;
        let node = state.node;
        let previous = state.current_visible;
        let value = visible.unwrap_or(state.baseline_visible);
        let expected_attribute = state.baseline_attribute.map(|_| value);
        let report = match self.surface.mutate_property(UiPropertyMutationRequest::new(
            node,
            "visible",
            UiValue::Bool(value),
        )) {
            Ok(report) => report,
            Err(error) => {
                self.visibility_state = None;
                return Outcome::RuntimeFailed(error);
            }
        };
        let publication = match report.status {
            UiPropertyMutationStatus::Accepted => {
                match self.surface.rebuild_dirty(self.preview_size) {
                    Ok(_) => Publication::Published(self.surface.surface_frame()),
                    Err(error) => Publication::Failed(error),
                }
            }
            UiPropertyMutationStatus::Unchanged | UiPropertyMutationStatus::Rejected => {
                Publication::NotRequested
            }
        };
        if report.status == UiPropertyMutationStatus::Rejected
            || matches!(&publication, Publication::Failed(_))
        {
            // Preserve the actual report even if acceptance preceded publication failure.
            self.visibility_state = None;
            return Outcome::Reported {
                report,
                publication,
            };
        }
        let proof = self.visibility_target_state(gesture.target());
        if !visibility_report_matches(&report, node, previous, value)
            || !proof
                .as_ref()
                .is_ok_and(|proof| proof.visible == value && proof.attribute == expected_attribute)
        {
            self.visibility_state = None;
            return Outcome::Reported {
                report,
                publication: Publication::VerificationFailed(self.surface.surface_frame()),
            };
        }
        if visible.is_none() {
            self.visibility_state = None;
        } else if let Some(state) = self.visibility_state.as_mut() {
            state.current_visible = value;
            state.current_attribute = expected_attribute;
            state.frame = self.surface.surface_frame();
        }
        Outcome::Reported {
            report,
            publication,
        }
    }

    /// Retire this lease while retaining its current value; no property write or publication.
    /// This does not restore an animation's preanimated state or implement Stop.
    pub(in crate::ui::asset_editor) fn finish_visibility_gesture(
        &mut self,
        gesture: &UiPreviewVisibilityGesture,
        source_frame: &Arc<UiSurfaceFrame>,
        sequence: u64,
    ) -> Result<Arc<UiSurfaceFrame>, Error> {
        self.admit_visibility_request(gesture, source_frame, sequence)?;
        self.retire_visibility_gesture(gesture);
        Ok(self.surface.surface_frame())
    }

    fn admit_visibility_request(
        &mut self,
        gesture: &UiPreviewVisibilityGesture,
        source_frame: &Arc<UiSurfaceFrame>,
        sequence: u64,
    ) -> Result<(), Error> {
        if let Err(error) = self.check_visibility_request(gesture, source_frame, sequence) {
            // An old callback must not retire a newer gesture or its valid frame.
            if !matches!(
                error,
                Error::RetiredGesture | Error::StaleRequest | Error::StaleFrame
            ) {
                self.retire_visibility_gesture(gesture);
            }
            return Err(error);
        }
        Ok(())
    }

    fn check_visibility_request(
        &self,
        gesture: &UiPreviewVisibilityGesture,
        source_frame: &Arc<UiSurfaceFrame>,
        sequence: u64,
    ) -> Result<(), Error> {
        let state = self
            .visibility_state
            .as_ref()
            .ok_or(Error::RetiredGesture)?;
        if !Arc::ptr_eq(&state.gesture.lease, &gesture.lease) {
            return Err(Error::RetiredGesture);
        }
        if sequence <= state.last_sequence {
            return Err(Error::StaleRequest);
        }
        let current = self.surface.surface_frame();
        if !Arc::ptr_eq(&state.frame, &current) {
            return Err(Error::FrameChanged);
        }
        if !Arc::ptr_eq(source_frame, &current) {
            return Err(Error::StaleFrame);
        }
        let proof = self.visibility_target_state(gesture.target())?;
        if proof.node != state.node
            || proof.incarnation != state.incarnation
            || proof.root != state.root
            || proof.root_incarnation != state.root_incarnation
            || proof.visible != state.current_visible
            || proof.attribute != state.current_attribute
        {
            return Err(Error::TargetChanged);
        }
        Ok(())
    }
}

/// A non-error return is insufficient: retain and verify the actual binding/dirty receipt.
fn visibility_report_matches(
    report: &UiPropertyMutationReport,
    node: UiNodeId,
    previous: bool,
    value: bool,
) -> bool {
    if report.node_id != node
        || report.property != "visible"
        || report.source != UiReflectedPropertySource::RuntimeState
        || report.binding.rejected_count != 0
    {
        return false;
    }
    let status = match report.status {
        UiPropertyMutationStatus::Accepted
            if previous != value
                && report.binding.applied_count > 0
                && report.invalidation.dirty.render
                && report.binding.dirty.contains(&UiBindingDirtyDomain::Render) =>
        {
            UiBindingUpdateStatus::Applied
        }
        UiPropertyMutationStatus::Unchanged
            if previous == value
                && report.binding.applied_count == 0
                && report.binding.unchanged_count > 0
                && !report.invalidation.any() =>
        {
            UiBindingUpdateStatus::Unchanged
        }
        _ => return false,
    };
    report.binding.updates.iter().any(|update| {
        update.status == status
            && update.source.kind == UiBindingSourceKind::RuntimeState
            && update.source.node_id == Some(node)
            && update.source.property.as_deref() == Some("visible")
            && update.target.node_id == Some(node)
            && update.target.property.as_deref() == Some("visible")
            && update.previous == Some(UiValue::Bool(previous))
            && update.value == UiValue::Bool(value)
    })
}

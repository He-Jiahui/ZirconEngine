//! A history-specific pre-publication DTO admission protocol. Backend scratch
//! and operation-task estimates are not a proof of Editor history memory.
use crate::core::framework::navigation::*;
use crate::operation::{RuntimeOperationHandlerError, RuntimeOperationPrepared};
use zircon_runtime_interface::retained_memory::*;

pub(in crate::navigation::operation) struct HistoryRefusalV1;
pub(in crate::navigation::operation) fn unwrap_request(
    value: serde_json::Value,
) -> Result<(NavigationHistoryBudgetV1, serde_json::Value), RuntimeOperationHandlerError> {
    let request: NavigationHistoryRequestV1 = serde_json::from_value(value).map_err(|e| {
        RuntimeOperationHandlerError::new(format!("invalid history admission envelope: {e}"))
    })?;
    if request.budget.version != 1
        || request.budget.retained_bytes == 0
        || request.budget.peak_bytes < request.budget.retained_bytes
    {
        return Err(RuntimeOperationHandlerError::new(
            "invalid history admission version or limits",
        ));
    }
    Ok((request.budget, request.payload))
}
pub(in crate::navigation::operation) fn certify_result(
    result: &mut serde_json::Value,
    budget: Option<NavigationHistoryBudgetV1>,
    before: &NavigationGeneratedBakeSnapshot,
    after: &NavigationGeneratedBakeSnapshot,
) -> Result<Option<RuntimeOperationPrepared>, RuntimeOperationHandlerError> {
    let Some(budget) = budget else {
        return Ok(None);
    };
    let Some(fields) = result.as_object_mut() else {
        return Err(RuntimeOperationHandlerError::new(
            "history result must be an object",
        ));
    };
    fields.insert(
        "_history_retention_v1".into(),
        serde_json::json!({"version":1, "accepted":true}),
    );
    let bound = budget.check_snapshots_v1(result, before, after);
    if bound.is_ok() {
        return Ok(None);
    }
    // Only this typed, versioned success outcome certifies "not published".
    // Generic Runtime errors and result byte limits keep their old semantics.
    Ok(Some(refusal_v1()))
}

pub(in crate::navigation::operation) fn preflight_snapshots(
    budget: Option<NavigationHistoryBudgetV1>,
    before: &NavigationGeneratedBakeSnapshot,
    after: &NavigationGeneratedBakeSnapshot,
    report: Option<&NavMeshBakeReport>,
) -> Result<Option<RuntimeOperationPrepared>, RuntimeOperationHandlerError> {
    let Some(budget) = budget else {
        return Ok(None);
    };
    if budget.check_projection_v1(before, after, report).is_ok() {
        return Ok(None);
    }
    Ok(Some(refusal_v1()))
}
fn refusal_v1() -> RuntimeOperationPrepared {
    // Closed tiny response and its zero-sized marker are included in the
    // protocol header allowance; no rejected DTO is copied into diagnostics.
    RuntimeOperationPrepared::with_owner_state(
        serde_json::Value::Null,
        serde_json::json!({"_history_retention_v1":{"version":1,"accepted":false,"reason":"prepublication_admission"}}),
        Box::new(HistoryRefusalV1),
        0,
    )
}

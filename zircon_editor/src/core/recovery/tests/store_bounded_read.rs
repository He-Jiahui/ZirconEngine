use std::io::Cursor;
use std::path::Path;

use zircon_runtime_interface::project::{
    ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
};

use super::{
    read_capped_ledger_bytes, ProjectSessionEffectLedger, ProjectSessionEffectLedgerStore,
    MAX_SESSION_EFFECT_LEDGER_BYTES,
};

#[test]
fn session_effect_ledger_reads_are_capped_before_deserialization() {
    let oversized = vec![b'x'; MAX_SESSION_EFFECT_LEDGER_BYTES * 4];
    let source = read_capped_ledger_bytes(Cursor::new(oversized)).expect("capped read");

    assert_eq!(source.len(), MAX_SESSION_EFFECT_LEDGER_BYTES + 1);
}

#[test]
fn decode_rejects_an_unreachable_closed_effect_inventory() {
    let operation_id = ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
        .allocate()
        .expect("fixture operation id");
    let ledger = ProjectSessionEffectLedger::for_operation(operation_id);
    let mut record = serde_json::to_value(ledger).expect("serialize fixture ledger");
    record["phase"] = serde_json::json!("closed");
    record["effects"] = serde_json::json!({ "runtime": "prepared" });
    let source = serde_json::to_vec(&record).expect("encode forged ledger");

    assert!(ProjectSessionEffectLedgerStore::decode(
        Path::new("forged-session-effect-ledger.json"),
        &source,
        operation_id,
    )
    .is_err());
}

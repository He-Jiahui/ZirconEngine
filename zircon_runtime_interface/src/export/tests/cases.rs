use super::{
    load_export_preset, ExportArtifactRef, ExportDigest, ExportPipelineReport, ExportPreset,
    ExportStage, ExportStageIo, ExportStageRecord, ExportStageStatus, ExportTargetMode,
};
use crate::serialization::write_versioned_text;

#[test]
fn export_stage_names_round_trip_across_cli_and_report_contracts() {
    for stage in ExportStage::ALL {
        assert_eq!(stage.cli_id().parse::<ExportStage>(), Ok(stage));
        assert_eq!(stage.report_name().parse::<ExportStage>(), Ok(stage));
    }
}

#[test]
fn export_preset_round_trips_through_the_versioned_text_envelope() {
    let preset = ExportPreset::new("desktop_windows", ExportTargetMode::ClientRuntime);
    let encoded = write_versioned_text(&preset).unwrap();
    let loaded = load_export_preset(encoded.as_bytes()).unwrap();

    assert!(encoded.contains("\"schema_id\": \"zircon.export-preset\""));
    assert_eq!(loaded, preset);
}

#[test]
fn export_preset_hard_cutover_rejects_unwrapped_version_zero_payloads() {
    let bytes = br#"{"profile_ref":"desktop_windows","target_mode":"client_runtime"}"#;
    assert!(matches!(
        load_export_preset(bytes),
        Err(super::ExportPresetLoadError::Envelope(_))
    ));
}

#[test]
fn export_preset_strict_loader_rejects_unknown_payload_fields() {
    let bytes = br#"{
        "$zircon": {
            "header": {"schema_id": "zircon.export-preset", "schema_version": 0},
            "payload": {
                "profile_ref": "desktop_windows",
                "target_mode": "client_runtime",
                "legacy_profile": "desktop"
            }
        }
    }"#;
    assert!(matches!(
        load_export_preset(bytes),
        Err(super::ExportPresetLoadError::Payload(_))
    ));
}

#[test]
fn export_preset_rejects_empty_profile_refs() {
    let preset = ExportPreset::new(" ", ExportTargetMode::ServerRuntime);
    assert!(matches!(
        preset.validate(),
        Err(super::ExportPresetValidationError::EmptyProfileRef)
    ));
}

#[test]
fn export_report_preserves_typed_stage_io_and_skip_status() {
    let digest = ExportDigest::from_bytes([7; 32]);
    let report = ExportPipelineReport {
        stages: vec![ExportStageRecord {
            stage: ExportStage::Pack,
            io: ExportStageIo {
                inputs: vec![ExportArtifactRef::new("cook", "cache/cook").with_digest(digest)],
                outputs: vec![ExportArtifactRef::new("pack", "dist/game.zrpack")],
                fingerprint: digest,
            },
            status: ExportStageStatus::Skipped,
            diagnostics: Vec::new(),
        }],
    };

    assert_eq!(
        report.record(ExportStage::Pack).unwrap().io.fingerprint,
        digest
    );
    assert_eq!(
        report.record(ExportStage::Pack).unwrap().status,
        ExportStageStatus::Skipped
    );
}

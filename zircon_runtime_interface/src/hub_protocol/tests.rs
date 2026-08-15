use std::path::PathBuf;
use std::str::FromStr;

use serde_json::json;

use super::{
    hub_editor_focus_signal_path, hub_recent_project_path_key, hub_recent_projects_lock_path,
    hub_recent_projects_path_from_home, merge_hub_recent_projects, HubEditorFocusSignalV1,
    HubEditorMailboxV1, HubProtocolVersionV1, HubRecentProjectV1, HubRecentProjectsV1,
    HubSessionToken, HUB_RECENT_PROJECT_LIMIT_V1,
};
use crate::project::{ProjectManifestSummary, PROJECT_MANIFEST_FORMAT_VERSION};

#[test]
fn hub_session_token_round_trips_as_a_uuid_without_path_characters() {
    let token = HubSessionToken::new();
    let encoded = token.to_string();

    assert_eq!(
        HubSessionToken::from_str(&encoded).expect("parse token"),
        token
    );
    assert_eq!(
        serde_json::from_str::<HubSessionToken>(
            &serde_json::to_string(&token).expect("serialize token")
        )
        .expect("deserialize token"),
        token
    );
    assert_eq!(encoded.len(), 36);
    assert!(encoded
        .chars()
        .all(|character| character.is_ascii_hexdigit() || character == '-'));
}

#[test]
fn hub_session_token_rejects_noncanonical_or_non_v4_values() {
    for invalid in [
        "../../not-a-session-token",
        "0D9A5890-0E44-4E2A-B77E-3E5D4FDF1E52",
        "0d9a58900e444e2ab77e3e5d4fdf1e52",
        "{0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52}",
        "urn:uuid:0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52",
        "0d9a5890-0e44-1e2a-b77e-3e5d4fdf1e52",
        "00000000-0000-0000-0000-000000000000",
    ] {
        assert!(
            HubSessionToken::from_str(invalid).is_err(),
            "must reject {invalid}"
        );
        assert!(
            serde_json::from_str::<HubSessionToken>(&format!("\"{invalid}\"")).is_err(),
            "serde must reject {invalid}"
        );
    }
}

#[test]
fn ready_mailbox_v1_round_trips_with_the_canonical_payload_shape() {
    let mailbox = HubEditorMailboxV1::ready(913, "E:/Projects/My Game");

    assert_eq!(mailbox.protocol_version, HubProtocolVersionV1);
    assert_eq!(
        serde_json::to_value(&mailbox).expect("serialize ready mailbox"),
        json!({
            "protocol_version": 1,
            "outcome": {
                "status": "ready",
                "pid": 913,
                "project": "E:/Projects/My Game",
            },
        })
    );
    assert_eq!(
        serde_json::from_value::<HubEditorMailboxV1>(json!({
            "protocol_version": 1,
            "outcome": {
                "status": "ready",
                "pid": 913,
                "project": "E:/Projects/My Game",
            },
        }))
        .expect("decode ready mailbox"),
        mailbox
    );
}

#[test]
fn stale_mailbox_protocol_is_rejected_at_the_decode_boundary() {
    let decoded = serde_json::from_value::<HubEditorMailboxV1>(json!({
        "protocol_version": 0,
        "outcome": {
            "status": "failed",
            "reason": "unsupported protocol",
        },
    }));

    assert!(decoded.is_err());
}

#[test]
fn failed_mailbox_v1_round_trips_with_the_canonical_payload_shape() {
    let mailbox = HubEditorMailboxV1::failed("project manifest is invalid");

    assert_eq!(
        serde_json::to_value(&mailbox).expect("serialize failed mailbox"),
        json!({
            "protocol_version": 1,
            "outcome": {
                "status": "failed",
                "reason": "project manifest is invalid",
            },
        })
    );
    assert_eq!(
        serde_json::from_value::<HubEditorMailboxV1>(json!({
            "protocol_version": 1,
            "outcome": {
                "status": "failed",
                "reason": "project manifest is invalid",
            },
        }))
        .expect("decode failed mailbox"),
        mailbox
    );
}

#[test]
fn mailbox_v1_rejects_legacy_or_extra_outcome_fields() {
    let decoded = serde_json::from_value::<HubEditorMailboxV1>(json!({
        "protocol_version": 1,
        "outcome": {
            "status": "ready",
            "pid": 913,
            "project": PathBuf::from("E:/Projects/My Game"),
            "legacy_project_path": "E:/Projects/My Game",
        },
    }));

    assert!(decoded.is_err());
}

#[test]
fn mailbox_v1_rejects_unknown_root_fields() {
    let decoded = serde_json::from_value::<HubEditorMailboxV1>(json!({
        "protocol_version": 1,
        "outcome": {
            "status": "failed",
            "reason": "project manifest is invalid",
        },
        "legacy_session": "retired",
    }));

    assert!(decoded.is_err());
}

#[test]
fn focus_signal_v1_round_trips_without_session_lock_fields() {
    let session = HubSessionToken::new();
    let signal = HubEditorFocusSignalV1::new(session, "913-1723718523000-1");

    assert_eq!(
        serde_json::to_value(&signal).expect("serialize focus signal"),
        json!({
            "protocol_version": 1,
            "session": session.to_string(),
            "target_instance_id": "913-1723718523000-1",
        })
    );
    assert_eq!(
        serde_json::from_value::<HubEditorFocusSignalV1>(json!({
            "protocol_version": 1,
            "session": session.to_string(),
            "target_instance_id": "913-1723718523000-1",
        }))
        .expect("deserialize focus signal"),
        signal
    );
}

#[test]
fn focus_signal_v1_rejects_unknown_or_legacy_lock_fields() {
    let decoded = serde_json::from_value::<HubEditorFocusSignalV1>(json!({
        "protocol_version": 1,
        "session": HubSessionToken::new().to_string(),
        "target_instance_id": "913-1723718523000-1",
        "process_id": 913,
    }));

    assert!(decoded.is_err());
}

#[test]
fn focus_signal_path_is_shared_and_rejects_an_unsafe_instance_id() {
    assert_eq!(
        hub_editor_focus_signal_path("E:/Projects/My Game", "913-1723718523000-1")
            .expect("valid instance id"),
        PathBuf::from("E:/Projects/My Game/.zircon/hub/focus/913-1723718523000-1.json")
    );
    assert!(hub_editor_focus_signal_path("E:/Projects/My Game", "../unsafe").is_err());
}

#[test]
fn recent_projects_v1_is_versioned_deduplicated_and_bounded() {
    let old = recent_project("Game", "E:/Projects/Game", 1);
    let current = recent_project("Game Updated", "e:\\Projects\\Game\\", 99);
    let additional = (0..10)
        .map(|index| {
            recent_project(
                format!("Project {index}"),
                format!("E:/Projects/Project{index}"),
                10 + index,
            )
        })
        .collect::<Vec<_>>();

    let registry = HubRecentProjectsV1::new(merge_hub_recent_projects(
        [old],
        additional.into_iter().chain([current]),
    ));

    assert_eq!(registry.projects.len(), HUB_RECENT_PROJECT_LIMIT_V1);
    assert_eq!(registry.projects[0].summary.name, "Game Updated");
    assert_eq!(
        registry
            .projects
            .iter()
            .filter(|project| hub_recent_project_path_key(&project.path) == "e:/projects/game")
            .count(),
        1
    );
    registry.validate().expect("canonical registry");
}

#[test]
fn recent_projects_v1_uses_the_shared_home_path_and_strict_wire_shape() {
    let registry = HubRecentProjectsV1::new([recent_project("Game", "E:/Projects/Game", 42)]);

    assert_eq!(
        hub_recent_projects_path_from_home("E:/Users/Zircon"),
        PathBuf::from("E:/Users/Zircon/.zircon/hub/recent_projects.json")
    );
    assert_eq!(
        hub_recent_projects_lock_path("E:/Users/Zircon/.zircon/hub/recent_projects.json"),
        PathBuf::from("E:/Users/Zircon/.zircon/hub/.recent_projects.json.lock")
    );
    assert_eq!(
        serde_json::to_value(&registry).expect("serialize shared registry"),
        json!({
            "protocol_version": 1,
            "projects": [{
                "summary": {
                    "name": "Game",
                    "engine_version_req": null,
                    "default_scene": "res://scenes/main.scene.toml",
                    "format_version": PROJECT_MANIFEST_FORMAT_VERSION,
                },
                "path": "E:/Projects/Game",
                "last_opened_unix_ms": 42,
            }],
        })
    );
    assert!(serde_json::from_value::<HubRecentProjectsV1>(json!({
        "protocol_version": 1,
        "projects": [],
        "legacy_session": true,
    }))
    .is_err());
}

fn recent_project(
    name: impl Into<String>,
    path: impl Into<PathBuf>,
    last_opened_unix_ms: u64,
) -> HubRecentProjectV1 {
    HubRecentProjectV1::new(
        ProjectManifestSummary {
            name: name.into(),
            engine_version_req: None,
            default_scene: "res://scenes/main.scene.toml".to_string(),
            format_version: PROJECT_MANIFEST_FORMAT_VERSION,
        },
        path,
        last_opened_unix_ms,
    )
    .expect("valid fixture recent project")
}

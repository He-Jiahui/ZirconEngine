//! Static contract for App-level localized error recovery feedback.

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_dir() -> PathBuf {
    crate_dir()
        .parent()
        .expect("zircon_hub should live below the repository root")
        .to_path_buf()
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path)).unwrap_or_else(|error| {
            panic!("failed to read Hub crate file {path}: {error}");
        }),
    )
}

fn read_repo_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(repo_dir().join(path)).unwrap_or_else(|error| {
            panic!("failed to read repository file {path}: {error}");
        }),
    )
}

fn assert_contains_all(source: &str, snippets: &[&str], label: &str) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{label} must contain App error-recovery snippet: {snippet}"
        );
    }
}

fn assert_not_contains_any(source: &str, snippets: &[&str], label: &str) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{label} must not expose obsolete App error-recovery snippet: {snippet}"
        );
    }
}

#[test]
fn app_error_recovery_copy_stays_on_the_localized_shell_dto() {
    let app = read_crate_file("web/src/App.tsx");
    let types = read_crate_file("web/src/types/hub.ts");
    let fallback = read_crate_file("web/src/data/hubData.ts");
    let ui_text = read_crate_file("src/tauri_app/view_model/ui_text.rs");
    let docs = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");

    assert_contains_all(
        &types,
        &[
            "liveUpdatesUnavailable: string;",
            "liveUpdatesUnavailableDetail: string;",
            "actionFailed: string;",
            "actionFailedDetail: string;",
            "stateRefreshAfterCommand: string;",
            "checkActionTarget: string;",
        ],
        "web/src/types/hub.ts",
    );
    assert_contains_all(
        &fallback,
        &[
            "liveUpdatesUnavailable: \"实时更新不可用\"",
            "liveUpdatesUnavailableDetail: \"无法订阅 Hub 状态更新。\"",
            "actionFailed: \"操作失败\"",
            "actionFailedDetail: \"Hub 后端未能完成该操作。\"",
            "stateRefreshAfterCommand: \"命令完成后仍会刷新状态\"",
            "checkActionTarget: \"检查操作目标后重试\"",
        ],
        "web/src/data/hubData.ts",
    );
    assert_contains_all(
        &ui_text,
        &[
            "pub live_updates_unavailable_detail: String",
            "pub action_failed_detail: String",
            ".pair(\n                    \"Unable to subscribe to Hub state updates.\",",
            ".pair(\n                    \"The Hub backend could not complete this action.\",",
        ],
        "src/tauri_app/view_model/ui_text.rs",
    );
    assert_contains_all(
        &app,
        &[
            "const stateRef = useRef(state);",
            "const shellText = stateRef.current.ui.shell;",
            "detail: shellText.liveUpdatesUnavailableDetail,",
            "recovery: shellText.stateRefreshAfterCommand,",
            "console.warn(shellText.liveUpdatesUnavailable, error);",
            "detail: shellText.actionFailedDetail,",
            "recovery: shellText.checkActionTarget,",
            "console.error(shellText.actionFailed, error);",
        ],
        "web/src/App.tsx",
    );
    assert_not_contains_any(
        &app,
        &["const detail = error instanceof Error ? error.message : String(error);"],
        "web/src/App.tsx",
    );
    assert_contains_all(
        &docs,
        &[
            "App-level subscription and command failures render localized `HubShellText` label/detail/recovery fields",
            "`liveUpdatesUnavailableDetail`",
            "`actionFailedDetail`",
        ],
        "docs/zircon_hub/ui/tauri-react-shell.md",
    );
}

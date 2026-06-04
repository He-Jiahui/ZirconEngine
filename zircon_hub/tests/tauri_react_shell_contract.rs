//! Static contracts for the Zircon Hub Tauri + React + Material UI shell.

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_crate_file(path: &str) -> String {
    fs::read_to_string(crate_dir().join(path))
        .map(|source| source.replace("\r\n", "\n"))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn assert_file(path: &str) {
    assert!(
        crate_dir().join(path).is_file(),
        "expected Zircon Hub file to exist: {path}"
    );
}

#[test]
fn tauri_shell_points_at_vite_react_frontend() {
    for path in [
        "tauri.conf.json",
        "package.json",
        "vite.config.ts",
        "web/index.html",
        "web/src/main.tsx",
        "web/src/App.tsx",
        "src/tauri_app.rs",
        "icons/icon.ico",
    ] {
        assert_file(path);
    }

    let tauri_config = read_crate_file("tauri.conf.json");
    for snippet in [
        "\"devUrl\": \"http://localhost:1420\"",
        "\"frontendDist\": \"web/dist\"",
        "\"beforeDevCommand\": \"npm run dev\"",
        "\"beforeBuildCommand\": \"npm run build\"",
        "\"decorations\": false",
        "\"icon\": [\"icons/icon.ico\"]",
        "\"width\": 1568",
        "\"height\": 1003",
    ] {
        assert!(
            tauri_config.contains(snippet),
            "tauri.conf.json must describe the fixed Hub window and Vite frontend handoff; missing {snippet}"
        );
    }

    let package_json = read_crate_file("package.json");
    for snippet in [
        "\"dev\": \"vite --host 127.0.0.1 --port 1420\"",
        "\"build\": \"tsc -b && vite build\"",
        "\"tauri:dev\": \"tauri dev\"",
        "\"@tauri-apps/api\": \"2.11.0\"",
        "\"@tauri-apps/cli\": \"2.11.2\"",
    ] {
        assert!(
            package_json.contains(snippet),
            "package.json must keep Vite/Tauri commands and packages aligned with tauri.conf.json; missing {snippet}"
        );
    }

    let tauri_app = read_crate_file("src/tauri_app.rs");
    for snippet in [
        "#[tauri::command]",
        "fn hub_state() -> HubShellState",
        "fn hub_action(request: HubActionRequest) -> HubShellState",
        "tauri::generate_handler![hub_state, hub_action]",
    ] {
        assert!(
            tauri_app.contains(snippet),
            "tauri_app.rs must expose a command boundary for React state loading and actions; missing {snippet}"
        );
    }
}

#[test]
fn react_material_components_are_split_from_low_level_to_window_shell() {
    for path in [
        "web/src/theme/tokens.ts",
        "web/src/theme/muiTheme.ts",
        "web/src/types/hub.ts",
        "web/src/data/hubData.ts",
        "web/src/tauri/hubApi.ts",
        "web/src/components/inputs/HubButton.tsx",
        "web/src/components/inputs/HubIconButton.tsx",
        "web/src/components/inputs/HubSearchField.tsx",
        "web/src/components/inputs/HubSelect.tsx",
        "web/src/components/inputs/HubToggle.tsx",
        "web/src/components/data/ProjectCard.tsx",
        "web/src/components/data/ProjectTable.tsx",
        "web/src/components/data/QuickActions.tsx",
        "web/src/components/data/StatusBadge.tsx",
        "web/src/components/overlays/HubMenu.tsx",
        "web/src/components/shell/NavigationDrawer.tsx",
        "web/src/components/shell/TopBar.tsx",
        "web/src/components/shell/HubWindow.tsx",
        "web/src/pages/ProjectsDashboard.tsx",
    ] {
        assert_file(path);
    }

    let inputs = read_crate_file("web/src/components/inputs/index.ts");
    for snippet in [
        "export * from \"./HubButton\";",
        "export * from \"./HubIconButton\";",
        "export * from \"./HubSearchField\";",
        "export * from \"./HubSelect\";",
        "export * from \"./HubToggle\";",
    ] {
        assert!(
            inputs.contains(snippet),
            "input layer must centralize low-level control exports; missing {snippet}"
        );
    }

    let data = read_crate_file("web/src/components/data/index.ts");
    for snippet in [
        "export * from \"./ProjectCard\";",
        "export * from \"./ProjectTable\";",
        "export * from \"./QuickActions\";",
        "export * from \"./StatusBadge\";",
        "export * from \"./HubPanel\";",
    ] {
        assert!(
            data.contains(snippet),
            "data-display layer must centralize card, table, list, panel, and badge exports; missing {snippet}"
        );
    }

    let shell = read_crate_file("web/src/components/shell/HubWindow.tsx");
    for snippet in [
        "<TopBar state={state} />",
        "<NavigationDrawer",
        "<ProjectsDashboard state={state} />",
    ] {
        assert!(
            shell.contains(snippet),
            "HubWindow must assemble only shell chrome and page regions; missing {snippet}"
        );
    }

    let page = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    for snippet in [
        "ProjectCard",
        "ProjectTable",
        "QuickActions",
        "HubSearchField",
        "HubSelect",
        "HubToggle",
        "ButtonStatesPanel",
    ] {
        assert!(
            page.contains(snippet),
            "ProjectsDashboard must compose shared components instead of page-local control markup; missing {snippet}"
        );
    }
}

#[test]
fn hub_visual_assets_are_runtime_assets_not_reference_screenshots() {
    let data = read_crate_file("web/src/data/hubData.ts");
    for snippet in [
        "../../../assets/brand/zircon-mark.svg",
        "../../../assets/covers/reference/project-elysium.png",
        "../../../assets/covers/reference/project-stellar-outpost.png",
        "../../../assets/covers/reference/project-sands-of-time.png",
        "../../../assets/covers/reference/project-whispering-woods.png",
    ] {
        assert!(
            data.contains(snippet),
            "React shell must use Hub runtime asset family for visible project media; missing {snippet}"
        );
    }

    for forbidden in [
        "docs/ui-and-layout/hub.png",
        "hub-ai-drafts",
        "hub-web-reference-1568x1003.png",
    ] {
        assert!(
            !data.contains(forbidden),
            "React shell must not render final/reference screenshots as runtime UI assets: {forbidden}"
        );
    }
}

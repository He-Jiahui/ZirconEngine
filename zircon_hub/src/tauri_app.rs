use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HubShellState {
    pub product_name: String,
    pub engine_version: String,
    pub active_page: String,
    pub task_status: Vec<HubStatusPill>,
    pub projects: Vec<HubProjectSummary>,
    pub recent_projects: Vec<HubRecentProject>,
    pub quick_actions: Vec<HubQuickAction>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HubStatusPill {
    pub id: String,
    pub label: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HubProjectSummary {
    pub id: String,
    pub name: String,
    pub path: String,
    pub modified: String,
    pub engine_version: String,
    pub platform: String,
    pub cover_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HubRecentProject {
    pub id: String,
    pub name: String,
    pub engine_version: String,
    pub modified: String,
    pub location: String,
    pub cover_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HubQuickAction {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub icon: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HubActionRequest {
    pub action_id: String,
    pub target_id: Option<String>,
}

#[tauri::command]
fn hub_state() -> HubShellState {
    reference_shell_state()
}

#[tauri::command]
fn hub_action(request: HubActionRequest) -> HubShellState {
    let _ = (request.action_id, request.target_id);
    reference_shell_state()
}

pub fn run() -> Result<(), crate::HubError> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![hub_state, hub_action])
        .run(tauri::generate_context!())?;
    Ok(())
}

fn reference_shell_state() -> HubShellState {
    HubShellState {
        product_name: "Zircon Hub".to_string(),
        engine_version: "Zircon Engine 1.8.2".to_string(),
        active_page: "projects".to_string(),
        task_status: vec![
            status("running", "Running", "running"),
            status("success", "Success", "success"),
            status("warning", "Warning", "warning"),
            status("error", "Error", "error"),
        ],
        projects: vec![
            project(
                "elysium",
                "Elysium Chronicles",
                r"C:\ZirconProjects\Elysium",
                "Modified 2h ago",
                "1.8.2",
                "Windows",
                "elysium",
            ),
            project(
                "stellar-outpost",
                "Stellar Outpost",
                r"C:\ZirconProjects\StellarOutpost",
                "Modified yesterday",
                "1.8.2",
                "Windows",
                "stellar-outpost",
            ),
            project(
                "sands-of-time",
                "Sands of Time",
                r"C:\ZirconProjects\SandsOfTime",
                "Modified 3d ago",
                "1.8.1",
                "Linux",
                "sands-of-time",
            ),
            project(
                "whispering-woods",
                "Whispering Woods",
                r"C:\ZirconProjects\WhisperingWoods",
                "Modified 1w ago",
                "1.8.0",
                "Windows",
                "whispering-woods",
            ),
        ],
        recent_projects: vec![
            recent(
                "elysium",
                "Elysium Chronicles",
                "1.8.2",
                "2h ago",
                r"C:\ZirconProjects\Elysium",
                "elysium",
            ),
            recent(
                "stellar-outpost",
                "Stellar Outpost",
                "1.8.2",
                "Yesterday",
                r"C:\ZirconProjects\StellarOutpost",
                "stellar-outpost",
            ),
            recent(
                "sands-of-time",
                "Sands of Time",
                "1.8.1",
                "3d ago",
                r"C:\ZirconProjects\SandsOfTime",
                "sands-of-time",
            ),
            recent(
                "whispering-woods",
                "Whispering Woods",
                "1.8.0",
                "1w ago",
                r"C:\ZirconProjects\WhisperingWoods",
                "whispering-woods",
            ),
            recent(
                "neon-streets",
                "Neon Streets",
                "1.7.9",
                "2w ago",
                r"C:\ZirconProjects\NeonStreets",
                "neon-streets",
            ),
        ],
        quick_actions: vec![
            quick_action(
                "build-project",
                "Build Project",
                "Build your project for development or release",
                "build",
            ),
            quick_action(
                "install-device",
                "Install to Device",
                "Deploy your project to a connected device",
                "device",
            ),
            quick_action(
                "package-project",
                "Package Project",
                "Create a distributable package",
                "package",
            ),
            quick_action(
                "open-editor",
                "Open in Editor",
                "Launch the editor with a project",
                "editor",
            ),
        ],
    }
}

fn status(id: &str, label: &str, tone: &str) -> HubStatusPill {
    HubStatusPill {
        id: id.to_string(),
        label: label.to_string(),
        tone: tone.to_string(),
    }
}

fn project(
    id: &str,
    name: &str,
    path: &str,
    modified: &str,
    engine_version: &str,
    platform: &str,
    cover_id: &str,
) -> HubProjectSummary {
    HubProjectSummary {
        id: id.to_string(),
        name: name.to_string(),
        path: path.to_string(),
        modified: modified.to_string(),
        engine_version: engine_version.to_string(),
        platform: platform.to_string(),
        cover_id: cover_id.to_string(),
    }
}

fn recent(
    id: &str,
    name: &str,
    engine_version: &str,
    modified: &str,
    location: &str,
    cover_id: &str,
) -> HubRecentProject {
    HubRecentProject {
        id: id.to_string(),
        name: name.to_string(),
        engine_version: engine_version.to_string(),
        modified: modified.to_string(),
        location: location.to_string(),
        cover_id: cover_id.to_string(),
    }
}

fn quick_action(id: &str, title: &str, detail: &str, icon: &str) -> HubQuickAction {
    HubQuickAction {
        id: id.to_string(),
        title: title.to_string(),
        detail: detail.to_string(),
        icon: icon.to_string(),
    }
}

use std::path::Path;

use slint::Image;

use crate::projects::{project_cover_path, RecentProject};
use crate::state::HubPage;

const HUB_ASSET_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");
const FALLBACK_COVERS: &[&str] = &[
    "covers/reference/project-elysium.png",
    "covers/reference/project-stellar-outpost.png",
    "covers/reference/project-sands-of-time.png",
    "covers/reference/project-whispering-woods.png",
    "covers/reference/project-neon-streets.png",
];

pub(super) fn navigation_icon(page: HubPage) -> Option<Image> {
    load_asset(match page {
        HubPage::Projects => "icons/nav/projects.svg",
        HubPage::Editor => "icons/nav/editor.svg",
        HubPage::Assets => "icons/nav/assets.svg",
        HubPage::Builds => "icons/nav/builds.svg",
        HubPage::Plugins => "icons/nav/plugins.svg",
        HubPage::Cloud => "icons/nav/cloud.svg",
        HubPage::Team => "icons/nav/team.svg",
        HubPage::Learn => "icons/nav/learn.svg",
        HubPage::Settings => "icons/nav/settings.svg",
    })
}

pub(super) fn quick_action_icon(action_id: &str) -> Option<Image> {
    load_asset(match action_id {
        "build-project" => "icons/actions/build-project.svg",
        "install-device" => "icons/actions/install-device.svg",
        "package-project" => "icons/actions/package-project.svg",
        "open-editor" => "icons/actions/open-editor.svg",
        _ => return None,
    })
}

pub(super) fn status_icon(state: &str) -> Option<Image> {
    load_asset(match state {
        "running" => "icons/status/running.svg",
        "ok" => "icons/status/success.svg",
        "warn" => "icons/status/warning.svg",
        "error" => "icons/status/error.svg",
        _ => return None,
    })
}

pub(super) fn project_cover(index: usize, project: &RecentProject) -> Option<Image> {
    load_project_cover_image(project).or_else(|| fallback_project_cover(index))
}

fn fallback_project_cover(index: usize) -> Option<Image> {
    if FALLBACK_COVERS.is_empty() {
        return None;
    }

    FALLBACK_COVERS
        .get(index % FALLBACK_COVERS.len())
        .and_then(|path| load_asset(path))
}

fn load_project_cover_image(project: &RecentProject) -> Option<Image> {
    if is_reference_fixture_project(project) {
        return None;
    }

    project_cover_path(&project.path).and_then(|path| Image::load_from_path(&path).ok())
}

fn is_reference_fixture_project(project: &RecentProject) -> bool {
    matches!(
        project.display_name.as_str(),
        "Elysium Chronicles"
            | "Stellar Outpost"
            | "Sands of Time"
            | "Whispering Woods"
            | "Neon Streets"
    ) && {
        let normalized = project.path.to_string_lossy().replace('\\', "/");
        normalized.starts_with("C:/ZirconProjects/") || normalized.contains("/C/ZirconProjects/")
    }
}

fn load_asset(relative_path: &str) -> Option<Image> {
    Image::load_from_path(&Path::new(HUB_ASSET_ROOT).join(relative_path)).ok()
}

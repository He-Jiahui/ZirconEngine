use super::*;

pub(super) fn project_overview_data(snapshot: &ProjectOverviewSnapshot) -> ProjectOverviewData {
    ProjectOverviewData {
        project_name: snapshot.project_name.clone().into(),
        project_root: snapshot.project_root.clone().into(),
        assets_root: snapshot.assets_root.clone().into(),
        library_root: snapshot.library_root.clone().into(),
        default_scene_uri: snapshot.default_scene_uri.clone().into(),
        catalog_revision: snapshot.catalog_revision.to_string().into(),
        folder_count: snapshot.folder_count.to_string().into(),
        asset_count: snapshot.asset_count.to_string().into(),
    }
}

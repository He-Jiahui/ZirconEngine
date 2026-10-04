use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedWorkbenchLayout {
    active_main_page: MainPageId,
    main_pages: Vec<MainHostPageLayout>,
    activity_windows: BTreeMap<ActivityWindowId, ActivityWindowLayout>,
    floating_windows: Vec<FloatingWindowLayout>,
}

impl<'de> Deserialize<'de> for WorkbenchLayout {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let saved = SavedWorkbenchLayout::deserialize(deserializer)?;
        let mut layout = Self {
            active_main_page: saved.active_main_page,
            main_pages: saved.main_pages,
            activity_windows: saved.activity_windows,
            floating_windows: saved.floating_windows,
        };
        layout.normalize_document_node_ids();
        Ok(layout)
    }
}

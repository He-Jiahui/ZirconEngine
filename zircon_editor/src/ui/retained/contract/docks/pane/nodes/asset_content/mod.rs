#[path = "../../../../../../contract/docks/pane/nodes/asset_content/projector.rs"]
mod projector;

pub(super) use projector::{ActivityAssetContentProjector, BrowserAssetContentProjector};

#[cfg(test)]
#[path = "../../../../workbench/docks/pane/nodes/tests/cases.rs"]
mod tests;

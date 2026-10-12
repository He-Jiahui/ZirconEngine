use super::*;

#[path = "asset_events/binding.rs"]
mod binding;
#[path = "asset_events/collection.rs"]
mod collection;
#[path = "asset_events/component_adapter.rs"]
mod component_adapter;
#[path = "asset_events/detail_dispatch.rs"]
mod detail_dispatch;
#[path = "asset_events/palette.rs"]
mod palette;
#[path = "asset_events/preview.rs"]
mod preview;
#[path = "asset_events/source.rs"]
mod source;
#[path = "asset_events/structure.rs"]
mod structure;
#[path = "asset_events/style.rs"]
mod style;
#[path = "asset_events/widget.rs"]
mod widget;

#[cfg(test)]
#[path = "tests/asset_events.rs"]
mod tests;

use std::sync::Arc;

use crate::core::framework::render::ViewportIconId;

use super::super::super::ViewportIconSource;
use super::super::{icon_entry::IconEntry, icon_slot::icon_slot};

pub(crate) struct ViewportIconAtlas {
    pub(super) source: Arc<dyn ViewportIconSource>,
    pub(super) entries: Vec<IconEntry>,
}

impl ViewportIconAtlas {
    pub(crate) fn new(source: Arc<dyn ViewportIconSource>) -> Self {
        Self {
            source,
            entries: vec![IconEntry::Unloaded; 2],
        }
    }

    pub(crate) fn has(&self, id: ViewportIconId) -> bool {
        matches!(self.entries[icon_slot(id)], IconEntry::Ready(_))
    }
}

mod journal;
mod state;

pub(crate) use journal::{RenderDirtyEntityJournal, RenderDirtyWorldId};
pub(super) use state::RenderDirtyJournalState;

use std::sync::Arc;

use crate::scene::World;

impl World {
    pub(crate) fn render_dirty_entity_journal(&self) -> Arc<RenderDirtyEntityJournal> {
        self.derived_state_dirty.render_dirty_entity_journal()
    }
}

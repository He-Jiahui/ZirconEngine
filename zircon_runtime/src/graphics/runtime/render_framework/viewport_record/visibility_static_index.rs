use crate::graphics::visibility::VisibilityStaticIndex;

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn visibility_static_index(
        &self,
    ) -> Option<&VisibilityStaticIndex> {
        self.visibility_static_index.as_ref()
    }

    pub(in crate::graphics::runtime::render_framework) fn replace_visibility_static_index(
        &mut self,
        index: VisibilityStaticIndex,
    ) {
        self.visibility_static_index = Some(index);
    }
}

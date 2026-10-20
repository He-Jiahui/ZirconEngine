use std::cell::RefCell;

use crate::ui::workbench::snapshot::{EditorChromeSnapshot, SceneEntries};

thread_local! {
    static HIERARCHY_ENTRIES: RefCell<Option<SceneEntries>> = const { RefCell::new(None) };
}

/// Pass the hierarchy projection independently of the much larger chrome snapshot.
/// Nested/native projections restore the previous input even when construction unwinds.
pub(crate) fn with_hierarchy_entries<T>(
    entries: Option<SceneEntries>,
    build: impl FnOnce() -> T,
) -> T {
    struct Restore(Option<SceneEntries>);
    impl Drop for Restore {
        fn drop(&mut self) {
            HIERARCHY_ENTRIES.with_borrow_mut(|entries| *entries = self.0.take());
        }
    }
    let _restore =
        Restore(HIERARCHY_ENTRIES.with_borrow_mut(|active| std::mem::replace(active, entries)));
    build()
}

pub(super) fn projected_hierarchy_entries(chrome: &EditorChromeSnapshot) -> SceneEntries {
    HIERARCHY_ENTRIES.with_borrow(|active| active.as_ref().unwrap_or(&chrome.scene_entries).clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_hierarchy_projection_restores_the_outer_input() {
        let outer = SceneEntries::default();
        let rows = outer.hierarchy_rows_arc();
        with_hierarchy_entries(Some(outer), || {
            with_hierarchy_entries(None, || {
                assert!(HIERARCHY_ENTRIES.with_borrow(Option::is_none))
            });
            assert!(
                HIERARCHY_ENTRIES.with_borrow(|entries| std::sync::Arc::ptr_eq(
                    &entries.as_ref().unwrap().hierarchy_rows_arc(),
                    &rows
                ))
            );
        });
        assert!(HIERARCHY_ENTRIES.with_borrow(Option::is_none));
    }
}

use super::{
    default_constraints_for_content, DEFAULT_CONSOLE_MIN_WIDTH, DEFAULT_CONSOLE_PREFERRED_WIDTH,
};
use crate::ui::workbench::snapshot::ViewContentKind;

#[test]
fn console_constraints_preserve_the_compact_filter_group_minimum() {
    let constraints = default_constraints_for_content(ViewContentKind::Console);

    assert_eq!(constraints.width.min, DEFAULT_CONSOLE_MIN_WIDTH);
    assert_eq!(constraints.width.preferred, DEFAULT_CONSOLE_PREFERRED_WIDTH);
    assert!(constraints.width.max < 0.0);
}

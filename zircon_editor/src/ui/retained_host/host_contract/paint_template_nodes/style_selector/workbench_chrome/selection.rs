use super::super::resolved_state_for_node;
use super::fill::chrome_fill;
use super::model::{WorkbenchChromeKind, WorkbenchChromeStyle};
use super::palette::{
    WORKBENCH_CHROME_SEPARATOR, WORKBENCH_CHROME_SOFT_SEPARATOR, WORKBENCH_CHROME_STRONG_SEPARATOR,
};
use super::separators::chrome_separator;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterFamily;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_chrome_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchChromeKind,
) -> WorkbenchChromeStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Chrome);

    WorkbenchChromeStyle {
        fill: chrome_fill(kind, state),
        separator: chrome_separator(WORKBENCH_CHROME_SEPARATOR, state),
        strong_separator: chrome_separator(WORKBENCH_CHROME_STRONG_SEPARATOR, state),
        soft_separator: chrome_separator(WORKBENCH_CHROME_SOFT_SEPARATOR, state),
        state,
    }
}

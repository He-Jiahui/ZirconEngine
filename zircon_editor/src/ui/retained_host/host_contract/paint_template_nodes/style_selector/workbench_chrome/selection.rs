use super::super::resolved_state_for_node;
use super::fill::chrome_fill;
use super::model::{WorkbenchChromeKind, WorkbenchChromeStyle};
use super::palette::workbench_chrome_palette;
use super::separators::chrome_separator;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterFamily;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_chrome_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchChromeKind,
) -> WorkbenchChromeStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Chrome);
    let palette = workbench_chrome_palette();

    WorkbenchChromeStyle {
        fill: chrome_fill(kind, state, &palette),
        separator: chrome_separator(palette.separator, state, &palette),
        strong_separator: chrome_separator(palette.strong_separator, state, &palette),
        soft_separator: chrome_separator(palette.soft_separator, state, &palette),
        state,
    }
}

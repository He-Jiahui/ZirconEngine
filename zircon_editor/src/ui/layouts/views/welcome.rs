use std::collections::BTreeMap;

use crate::ui::layouts::views::view_projection::build_view_template_node_projection;
use crate::ui::retained_host::primitives::ModelRc;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const WELCOME_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/welcome.zui";

pub(crate) fn welcome_pane_nodes(size: UiSize) -> ModelRc<ViewTemplateNodeData> {
    // This bootstrap projection precedes the workbench's localization context.
    // Resolve its authored references through the same canonical catalog.
    let text = BTreeMap::from(
        [
            ("WelcomeOpenExistingButton", "editor.welcome.open_project"),
            (
                "WelcomeCreateProjectButton",
                "editor.welcome.create_project",
            ),
        ]
        .map(|(control, key)| {
            (
                control.to_owned(),
                crate::ui::retained_host::startup_splash::content::startup_text(key).to_string(),
            )
        }),
    );
    build_view_template_node_projection(
        "welcome.template_projection",
        WELCOME_LAYOUT_ASSET_PATH,
        &[],
        size,
        &text,
    )
    .map(|projection| projection.into_model())
    .unwrap_or_default()
}

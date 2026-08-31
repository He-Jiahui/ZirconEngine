use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession};
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

const PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.preview_binding"
version = 1
display_name = "Preview Binding"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { text = "Ready" }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", text_expr = "=status.text" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "menu_action.workbench.project.save" }]
"##;

const PREVIEW_STATE_GRAPH_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.preview_state_graph"
version = 1
display_name = "Preview State Graph"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { metadata = { title = "Ready", count = 1 }, items = ["Ready", "Dirty"] }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", text_expr = "=StatusLabel.metadata.title" }
"##;

const PREVIEW_BRACKET_EXPRESSION_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.preview_bracket_expression"
version = 1
display_name = "Preview Bracket Expression"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { items = ["Ready", "Dirty"] }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", item_expr = "=StatusLabel.items[1]" }
"##;

const PREVIEW_DEEP_NESTED_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.preview_deep_nested"
version = 1
display_name = "Preview Deep Nested"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { context = { dialog = { title = "Ready", steps = [{ label = "Plan" }, { label = "Dirty" }] } } }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", text_expr = "=StatusLabel.context.dialog.steps[1].label" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "route.form.value_changed" }]
"##;

const PREVIEW_FUNCTION_EXPRESSION_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.preview_function_expression"
version = 1
display_name = "Preview Function Expression"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { text = "Ready", subtitle = "", items = ["Ready", "Dirty"], metadata = { title = "Ready", severity = "Info" } }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", summary_expr = "=concat(StatusLabel.text, \" / \", self.text)", fallback_expr = "=coalesce(StatusLabel.subtitle, StatusLabel.text, \"Unknown\")", item_count_expr = "=count(StatusLabel.items)", first_item_expr = "=first(StatusLabel.items)", last_item_expr = "=last(StatusLabel.items)", joined_items_expr = "=join(StatusLabel.items, \" | \")", status_matches_expr = "=eq(StatusLabel.text, \"Dirty\")", cta_expr = "=if(eq(StatusLabel.text, \"Dirty\"), \"Go\", \"Stop\")", metadata_title_expr = "=get(StatusLabel.metadata, \"title\")", review_item_expr = "=at(StatusLabel.items, 1)", has_title_expr = "=has(StatusLabel.metadata, \"title\")" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "menu_action.workbench.project.save" }]
"##;

mod expression_evaluation;
mod nested_authoring;
mod payload_paths;
mod schema_projection;

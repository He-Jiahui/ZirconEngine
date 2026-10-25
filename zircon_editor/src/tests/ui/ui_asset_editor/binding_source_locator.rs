use crate::ui::asset_editor::{
    UiAssetBindingSourceError as Error, UiAssetBindingSourceScope as Scope,
    UiAssetBindingSourceSelector, UiAssetEditorCommand, UiAssetSourceOwner, UiAssetSourceRange,
};
use zircon_runtime_interface::ui::binding::UiEventKind;

use super::support::open_design_session;

const URI: &str = "editor.test.binding_source";
const SOURCE: &str = r#"[asset]
kind = "view"
id = "editor.test.binding_source"
version = 2
[root]
node = "root"
[nodes.root]
component = "Panel"
children = [{ node = "按钮.😀" }]
[nodes."按钮.😀"]
component = "Button"
# id = "click" in a comment is not a declaration.
events = [
    { id = "click", event = "Click", route = "editor.test.click" },
    { id = "focus", event = "Focus" },
]
"#;

fn selector() -> UiAssetBindingSourceSelector {
    UiAssetBindingSourceSelector {
        node_id: "按钮.😀".to_owned(),
        binding_id: "click".to_owned(),
        event: UiEventKind::Click,
        component_event: None,
        owner: None,
    }
}

fn slice<'a>(source: &'a str, range: &UiAssetSourceRange) -> &'a str {
    &source[range.start.byte_offset..range.end.byte_offset]
}

#[test]
fn inline_binding_span_is_exact_and_preserves_original_declaration() {
    let source = SOURCE.replace('\n', "\r\n");
    let mut session = open_design_session(URI, &source);
    let location = session
        .locate_binding_source(&selector(), Scope::Current)
        .unwrap();
    let declaration = location.declaration();
    assert_eq!(slice(&source, &declaration.id), "\"click\"");
    assert_eq!(slice(&source, &declaration.event), "\"Click\"");
    assert_eq!(
        slice(&source, &declaration.declaration),
        "{ id = \"click\", event = \"Click\", route = \"editor.test.click\" }"
    );
    assert_eq!(declaration.declaration.start.line, 14);
    assert_eq!(declaration.declaration.start.utf16_column, 4);
    assert_eq!(
        declaration.owner,
        UiAssetSourceOwner::AssetRoot {
            root_node_id: "root".into()
        }
    );
    assert_eq!(declaration.control_component, "Button");
    assert_eq!(
        declaration.binding.route.as_deref(),
        Some("editor.test.click")
    );
    assert_eq!(
        location.range_source_digest(),
        blake3::hash(source.as_bytes())
    );
    assert_eq!(location.scope(), Scope::Current);
    assert_eq!(location.request_buffer_revision(), 0);
    assert_eq!(location.last_valid_generation(), 0);
    assert!(session.select_binding_source_location(&location).unwrap());
    let pane = session.pane_presentation();
    assert_eq!(
        pane.source_cursor_byte_offset as usize,
        declaration.declaration.start.byte_offset
    );
    assert_eq!(pane.inspector_selected_node_id, "按钮.😀");
    assert_eq!(session.source_buffer().text(), source);
    assert!(!session.source_buffer().is_dirty());
    assert!(!session.can_undo());
    assert!(!session.can_redo());
    assert!(!session.select_binding_source_location(&location).unwrap());
}

#[test]
fn array_table_binding_uses_its_own_id_and_event_spans() {
    let source = SOURCE.split("events = [").next().unwrap().to_owned()
        + r#"
[[nodes."按钮.😀".events]]
id = "click"
event = "Click"
route = "editor.test.click"
[[nodes."按钮.😀".events]]
id = "focus"
event = "Focus"
"#;
    let session = open_design_session(URI, &source);
    let location = session
        .locate_binding_source(&selector(), Scope::Current)
        .unwrap();
    assert_eq!(slice(&source, &location.declaration().id), "\"click\"");
    assert_eq!(slice(&source, &location.declaration().event), "\"Click\"");
    let declaration = slice(&source, &location.declaration().declaration);
    assert!(declaration.starts_with("[[nodes.\"按钮.😀\".events]]"));
    assert!(declaration.ends_with("route = \"editor.test.click\""));
    assert!(!declaration.contains("id = \"focus\""));
    assert!(location.declaration().id.start.byte_offset < source.find("id = \"focus\"").unwrap());
}

#[test]
fn invalid_current_source_is_distinct_from_read_only_last_valid_source() {
    let mut session = open_design_session(URI, SOURCE);
    let old = session
        .locate_binding_source(&selector(), Scope::Current)
        .unwrap();
    session
        .apply_command(UiAssetEditorCommand::edit_source(format!(
            "{SOURCE}\n[broken"
        )))
        .unwrap();
    assert!(matches!(
        session.locate_binding_source(&selector(), Scope::Current),
        Err(Error::InvalidCurrentSource)
    ));
    let last_valid = session
        .locate_binding_source(&selector(), Scope::LastValid)
        .unwrap();
    assert_eq!(last_valid.scope(), Scope::LastValid);
    assert_eq!(
        last_valid.range_source_digest(),
        blake3::hash(SOURCE.as_bytes())
    );
    assert!(last_valid.request_buffer_revision() > old.request_buffer_revision());
    assert_eq!(
        last_valid.last_valid_generation(),
        old.last_valid_generation()
    );
    let before = session.pane_presentation().source_cursor_byte_offset;
    assert_eq!(
        session.select_binding_source_location(&last_valid),
        Err(Error::LastValidReadOnly)
    );
    assert_eq!(
        session.select_binding_source_location(&old),
        Err(Error::StaleLocation)
    );
    assert_eq!(
        session.pane_presentation().source_cursor_byte_offset,
        before
    );
    assert!(session.source_buffer().text().ends_with("[broken"));
}

#[test]
fn source_edit_undo_and_reopen_retire_old_locations() {
    let mut session = open_design_session(URI, SOURCE);
    let old = session
        .locate_binding_source(&selector(), Scope::Current)
        .unwrap();
    session
        .apply_command(UiAssetEditorCommand::edit_source(format!(
            "# newer\n{SOURCE}"
        )))
        .unwrap();
    assert_eq!(
        session.select_binding_source_location(&old),
        Err(Error::StaleLocation)
    );
    assert!(session.undo_replay().unwrap().changed);
    assert_eq!(session.source_buffer().text(), SOURCE);
    assert_eq!(
        session.select_binding_source_location(&old),
        Err(Error::StaleLocation)
    );
    let current = session
        .locate_binding_source(&selector(), Scope::Current)
        .unwrap();
    let mut reopened = open_design_session(URI, SOURCE);
    assert_eq!(
        reopened.select_binding_source_location(&current),
        Err(Error::StaleLocation)
    );
    assert!(session.select_binding_source_location(&current).is_ok());
}

#[test]
fn missing_event_and_owner_mismatch_remain_distinct() {
    let session = open_design_session(URI, SOURCE);
    let mut query = selector();
    query.binding_id = "absent".into();
    assert!(matches!(
        session.locate_binding_source(&query, Scope::Current),
        Err(Error::MissingBinding { .. })
    ));
    query = selector();
    query.event = UiEventKind::Change;
    assert!(matches!(
        session.locate_binding_source(&query, Scope::Current),
        Err(Error::EventMismatch)
    ));
    query = selector();
    query.owner = Some(UiAssetSourceOwner::Component {
        name: "Button".into(),
        root_node_id: "按钮.😀".into(),
    });
    assert!(matches!(
        session.locate_binding_source(&query, Scope::Current),
        Err(Error::OwnerMismatch)
    ));
}

#[test]
fn repeated_component_calls_do_not_change_the_declarations_owning_component() {
    let source = SOURCE.replace(
        "children = [{ node = \"按钮.😀\" }]",
        "children = [{ node = \"first\" }, { node = \"second\" }]",
    ) + r#"
[nodes.first]
component = "Card"
[nodes.second]
component = "Card"
[nodes.card_root]
component = "Panel"
children = [{ node = "按钮.😀" }]
[components.Card]
root = "card_root"
"#;
    let session = open_design_session(URI, &source);
    let location = session
        .locate_binding_source(&selector(), Scope::Current)
        .unwrap();
    assert_eq!(
        location.declaration().owner,
        UiAssetSourceOwner::Component {
            name: "Card".into(),
            root_node_id: "card_root".into(),
        }
    );
    assert_eq!(location.declaration().control_component, "Button");
    assert_eq!(location.declaration().binding.id, "click");
}

#[test]
fn a_preview_compile_failure_does_not_hide_valid_source_declarations() {
    let source = SOURCE
        .replace(
            "component = \"Button\"",
            "component = \"UnavailableWidget\"",
        )
        .replace(
            "event = \"Click\", route",
            "event = \"Click\", component_event = \"Press\", route",
        );
    let mut session = open_design_session(URI, &source);
    assert!(
        !session.diagnostics().is_empty(),
        "actual typed-event compilation requires a known component descriptor"
    );
    let mut query = selector();
    query.component_event =
        Some(zircon_runtime_interface::ui::component::UiComponentEventKind::Press);
    let location = session
        .locate_binding_source(&query, Scope::Current)
        .unwrap();
    assert_eq!(slice(&source, &location.declaration().id), "\"click\"");
    assert_eq!(
        location.declaration().control_component,
        "UnavailableWidget"
    );
    assert!(session.select_binding_source_location(&location).unwrap());
    assert_eq!(session.source_buffer().text(), source);
    assert!(!session.can_undo());
}

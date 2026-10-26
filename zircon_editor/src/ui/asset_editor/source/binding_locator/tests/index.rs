    use super::*;
    use crate::ui::asset_editor::source::binding_locator::UiAssetSourceOwner;
    use zircon_runtime_interface::ui::binding::UiEventKind;

    const SOURCE: &str = r#"[asset]
kind = "view"
id = "test.binding_spans"
version = 2
[root]
node = "root"
[nodes.root]
component = "Panel"
children = [{ node = "button" }]
[nodes.button]
component = "Button"
events = [{ id = "click", event = "Click" }]
"#;

    fn selector() -> UiAssetBindingSourceSelector {
        UiAssetBindingSourceSelector {
            node_id: "button".into(),
            binding_id: "click".into(),
            event: UiEventKind::Click,
            component_event: None,
            owner: None,
        }
    }

    #[test]
    fn duplicate_binding_ids_are_ambiguous_even_with_different_events() {
        let source = SOURCE.replace("events = [{ id = \"click\", event = \"Click\" }]",
            "events = [{ id = \"click\", event = \"Click\" }, { id = \"click\", event = \"Focus\" }]");
        let document = toml::from_str(&source).unwrap();
        assert_eq!(
            UiAssetBindingSourceIndex::parse(&source).locate(&source, &document, &selector()),
            Err(Error::AmbiguousBinding {
                binding_id: "click".into(),
                count: 2
            })
        );
    }

    #[test]
    fn ambiguous_and_unattached_source_owners_are_reported_without_guessing() {
        let source = SOURCE.to_owned() + "\n[components.Shared]\nroot = \"button\"\n";
        let document = toml::from_str(&source).unwrap();
        let index = UiAssetBindingSourceIndex::parse(&source);
        let Err(Error::AmbiguousOwner { owners }) = index.locate(&source, &document, &selector())
        else {
            panic!("both declaration roots must be retained");
        };
        assert_eq!(owners.len(), 2);
        let mut query = selector();
        query.owner = Some(UiAssetSourceOwner::Component {
            name: "Shared".into(),
            root_node_id: "button".into(),
        });
        assert_eq!(
            index.locate(&source, &document, &query).unwrap().owner,
            query.owner.unwrap()
        );
        let source = SOURCE.replace("children = [{ node = \"button\" }]", "children = []");
        let document = toml::from_str(&source).unwrap();
        assert_eq!(
            UiAssetBindingSourceIndex::parse(&source).locate(&source, &document, &selector()),
            Err(Error::MissingOwner)
        );
    }

    #[test]
    fn a_cached_index_rejects_different_text_and_a_changed_binding_projection() {
        let document: UiV2AssetDocument = toml::from_str(SOURCE).unwrap();
        let index = UiAssetBindingSourceIndex::parse(SOURCE);
        let changed_source = format!("# changed\n{SOURCE}");
        assert_eq!(
            index.locate(&changed_source, &document, &selector()),
            Err(Error::SourceMismatch)
        );
        let mut changed_document = document;
        changed_document.nodes.get_mut("button").unwrap().events[0].event = UiEventKind::Focus;
        assert_eq!(
            index.locate(SOURCE, &changed_document, &selector()),
            Err(Error::SourceMismatch)
        );
        changed_document.nodes.get_mut("button").unwrap().events[0].event = UiEventKind::Click;
        changed_document.nodes.get_mut("button").unwrap().events[0].route =
            Some("changed.route".into());
        assert_eq!(
            index.locate(SOURCE, &changed_document, &selector()),
            Err(Error::SourceMismatch)
        );
    }

    #[test]
    fn array_table_range_includes_nested_payload_and_retains_typed_event_and_targets() {
        let source = SOURCE.split("events = [").next().unwrap().to_owned()
            + r#"
[[nodes.button.events]]
id = "click"
event = "Click"
component_event = "Press"
targets = [{ target = { kind = "visibility" }, expression = "true" }]
[nodes.button.events.action]
route = "editor.test.click"
[nodes.button.events.action.payload]
message = """中文😀
long text"""
[[nodes.button.events]]
id = "focus"
event = "Focus"
"#;
        let document = toml::from_str(&source).unwrap();
        let mut query = selector();
        query.component_event =
            Some(zircon_runtime_interface::ui::component::UiComponentEventKind::Press);
        let declaration = UiAssetBindingSourceIndex::parse(&source)
            .locate(&source, &document, &query)
            .unwrap();
        let text = &source
            [declaration.declaration.start.byte_offset..declaration.declaration.end.byte_offset];
        assert!(text.starts_with("[[nodes.button.events]]"));
        assert!(text.ends_with("long text\"\"\""));
        assert!(!text.contains("id = \"focus\""));
        let component_event = declaration.component_event.unwrap();
        assert_eq!(
            &source[component_event.start.byte_offset..component_event.end.byte_offset],
            "\"Press\""
        );
        assert_eq!(declaration.binding.targets.len(), 1);
        let action = declaration.binding.action.unwrap();
        assert_eq!(action.route.as_deref(), Some("editor.test.click"));
        assert_eq!(
            action.payload.get("message").and_then(toml::Value::as_str),
            Some("中文😀\nlong text")
        );
    }

use std::collections::{BTreeMap, BTreeSet};

use super::support::{collect_zui_document_files, editor_asset_root, load_zui_document};

const WORKBENCH_STYLESHEETS: &[&str] = &[
    "ui/theme/editor_workbench_strict.zui",
    "ui/theme/editor_workbench_spatial.zui",
];
const SUPPORTED_WORKBENCH_SELECTOR_STATES: &[&str] = &[
    "checked",
    "disabled",
    "focus-visible",
    "hovered",
    "popup_open",
    "pressed",
    "selected",
];
#[test]
fn workbench_theme_selectors_are_unique_and_have_an_authored_class_source() {
    let asset_root = editor_asset_root();
    let authored_classes = collect_authored_classes();

    for stylesheet_locator in WORKBENCH_STYLESHEETS {
        let stylesheet_path = asset_root.join(stylesheet_locator);
        let stylesheet = load_zui_document(&stylesheet_path);
        let mut selector_counts = BTreeMap::new();
        let mut unreachable = BTreeSet::new();
        let mut offenders = Vec::new();

        for sheet in &stylesheet.stylesheets {
            for rule in &sheet.rules {
                *selector_counts
                    .entry(rule.selector.as_str())
                    .or_insert(0usize) += 1;
                let Some(base_class) = selector_base_class(&rule.selector) else {
                    offenders.push(format!(
                        "selector `{}` is outside the governed `.class[:state]` vocabulary",
                        rule.selector
                    ));
                    continue;
                };
                if !authored_classes.contains(base_class) {
                    unreachable.insert(rule.selector.as_str());
                }
            }
        }

        for (selector, count) in selector_counts {
            if count > 1 {
                offenders.push(format!(
                    "selector `{selector}` is declared {count} times; merge the duplicate state recipe"
                ));
            }
        }

        assert!(
            unreachable.is_empty(),
            "{stylesheet_locator} selectors need a node class or component default_classes source: {unreachable:#?}"
        );
        assert!(
            offenders.is_empty(),
            "{stylesheet_locator} selectors must stay unique and use the governed selector vocabulary: {offenders:#?}"
        );
    }
}

#[test]
fn selector_base_class_parses_the_governed_vocabulary() {
    assert_eq!(
        selector_base_class(".workbench-field:focus-visible"),
        Some("workbench-field")
    );
    assert_eq!(
        selector_base_class(".workbench-primitive"),
        Some("workbench-primitive")
    );
    assert_eq!(selector_base_class("#workbench-field"), None);
    assert_eq!(selector_base_class(".workbench field"), None);
    assert_eq!(selector_base_class(".workbench-field:typo_state"), None);
    assert_eq!(
        selector_base_class(".workbench-field:hovered:pressed"),
        None
    );
}

fn collect_authored_classes() -> BTreeSet<String> {
    let editor_ui_root = editor_asset_root().join("ui/editor");
    let mut classes = BTreeSet::new();
    for path in collect_zui_document_files(&editor_ui_root) {
        let document = load_zui_document(&path);
        for node in document.nodes.values() {
            classes.extend(node.classes.iter().cloned());
        }
        for component in document.components.values() {
            classes.extend(component.default_classes.iter().cloned());
        }
    }
    classes
}

fn selector_base_class(selector: &str) -> Option<&str> {
    let selector = selector.strip_prefix('.')?;
    let mut parts = selector.split(':');
    let base_class = parts.next()?;
    let state = parts.next();
    if parts.next().is_some()
        || !is_selector_word(base_class)
        || state.is_some_and(|state| {
            !is_selector_word(state) || !SUPPORTED_WORKBENCH_SELECTOR_STATES.contains(&state)
        })
    {
        return None;
    }
    Some(base_class)
}

fn is_selector_word(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

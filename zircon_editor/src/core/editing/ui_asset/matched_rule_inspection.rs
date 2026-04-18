use std::collections::BTreeMap;

use crate::ui::UiMatchedStyleRuleReflection;
use zircon_ui::{UiAssetDocument, UiNodeDefinition, UiStyleSheet};

use super::style_rule_declarations::{declaration_entries, UiStyleRuleDeclarationEntry};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MatchedStyleRuleEntry {
    pub(crate) origin_id: String,
    pub(crate) selector: String,
    pub(crate) specificity: usize,
    pub(crate) source_order: usize,
    pub(crate) declarations: Vec<UiStyleRuleDeclarationEntry>,
}

impl MatchedStyleRuleEntry {
    pub(crate) fn reflection(&self) -> UiMatchedStyleRuleReflection {
        UiMatchedStyleRuleReflection::new(
            self.origin_id.clone(),
            self.selector.clone(),
            self.specificity,
            self.source_order,
        )
    }

    pub(crate) fn label(&self) -> String {
        format!("{} [{}]", self.selector, self.origin_id)
    }

    pub(crate) fn declaration_items(&self) -> Vec<String> {
        self.declarations
            .iter()
            .map(|entry| format!("{} = {}", entry.path, entry.literal))
            .collect()
    }
}

pub(crate) fn selector_is_valid(selector: &str) -> bool {
    InspectorSelector::parse(selector).is_some()
}

pub(crate) fn matched_style_rule_entries(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    node_id: &str,
    active_states: &[String],
) -> Vec<MatchedStyleRuleEntry> {
    let Some(path) = document_node_path(document, node_id) else {
        return Vec::new();
    };
    let match_path = path
        .iter()
        .enumerate()
        .map(|(index, (_, node))| StyleMatchNode {
            component: selector_component_name(node),
            control_id: node.control_id.as_deref(),
            classes: &node.classes,
            is_host: index == 0,
            states: active_states,
        })
        .collect::<Vec<_>>();

    let mut matched = Vec::new();
    let mut order = 0;
    for reference in &document.imports.styles {
        if let Some(imported) = imported_styles.get(reference) {
            collect_matching_rules(
                &mut matched,
                &match_path,
                &imported.asset.id,
                &imported.stylesheets,
                &mut order,
            );
        }
    }
    collect_matching_rules(
        &mut matched,
        &match_path,
        &document.asset.id,
        &document.stylesheets,
        &mut order,
    );
    matched.sort_by_key(|rule| (rule.specificity, rule.source_order));
    matched
}

fn collect_matching_rules(
    output: &mut Vec<MatchedStyleRuleEntry>,
    path: &[StyleMatchNode<'_>],
    origin: &str,
    stylesheets: &[UiStyleSheet],
    order: &mut usize,
) {
    for stylesheet in stylesheets {
        let origin_id = if stylesheet.id.is_empty() {
            origin.to_string()
        } else {
            format!("{origin}::{}", stylesheet.id)
        };
        for rule in &stylesheet.rules {
            if let Some(selector) = InspectorSelector::parse(&rule.selector) {
                if selector.matches_path(path) {
                    output.push(MatchedStyleRuleEntry {
                        origin_id: origin_id.clone(),
                        selector: rule.selector.clone(),
                        specificity: selector.specificity(),
                        source_order: *order,
                        declarations: declaration_entries(&rule.set),
                    });
                }
            }
            *order += 1;
        }
    }
}

fn document_node_path<'a>(
    document: &'a UiAssetDocument,
    node_id: &str,
) -> Option<Vec<(&'a str, &'a UiNodeDefinition)>> {
    fn visit<'a>(
        document: &'a UiAssetDocument,
        current_id: &'a str,
        target: &str,
        path: &mut Vec<(&'a str, &'a UiNodeDefinition)>,
    ) -> bool {
        let Some(node) = document.nodes.get(current_id) else {
            return false;
        };
        path.push((current_id, node));
        if current_id == target {
            return true;
        }
        for child in &node.children {
            if visit(document, child.child.as_str(), target, path) {
                return true;
            }
        }
        let _ = path.pop();
        false
    }

    let root = document.root.as_ref()?;
    let mut path = Vec::new();
    visit(document, root.node.as_str(), node_id, &mut path).then_some(path)
}

pub(crate) fn selector_component_name(node: &UiNodeDefinition) -> &str {
    node.widget_type
        .as_deref()
        .or_else(|| node.component.as_deref())
        .or_else(|| {
            node.component_ref
                .as_deref()
                .and_then(|reference| reference.split_once('#').map(|(_, component)| component))
        })
        .unwrap_or("Node")
}

#[derive(Clone, Copy)]
struct StyleMatchNode<'a> {
    component: &'a str,
    control_id: Option<&'a str>,
    classes: &'a [String],
    is_host: bool,
    states: &'a [String],
}

#[derive(Clone)]
struct InspectorSelector {
    segments: Vec<InspectorSelectorSegment>,
}

#[derive(Clone)]
struct InspectorSelectorSegment {
    combinator: Option<InspectorSelectorCombinator>,
    tokens: Vec<InspectorSelectorToken>,
}

#[derive(Clone, Copy)]
enum InspectorSelectorCombinator {
    Descendant,
    Child,
}

#[derive(Clone)]
enum InspectorSelectorToken {
    Type(String),
    Class(String),
    Id(String),
    State(String),
    Host,
}

impl InspectorSelector {
    fn parse(input: &str) -> Option<Self> {
        let mut chars = input.chars().peekable();
        let mut segments = Vec::new();
        let mut combinator = None;

        loop {
            let saw_whitespace = skip_selector_whitespace(&mut chars);
            if chars.peek().is_none() {
                if saw_whitespace && segments.is_empty() {
                    return None;
                }
                break;
            }

            let mut compound = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() || ch == '>' {
                    break;
                }
                compound.push(ch);
                let _ = chars.next();
            }

            if compound.is_empty() {
                return None;
            }

            segments.push(InspectorSelectorSegment {
                combinator,
                tokens: parse_selector_tokens(&compound)?,
            });

            let saw_space = skip_selector_whitespace(&mut chars);
            combinator = match chars.peek().copied() {
                Some('>') => {
                    let _ = chars.next();
                    Some(InspectorSelectorCombinator::Child)
                }
                Some(_) if saw_space => Some(InspectorSelectorCombinator::Descendant),
                Some(_) => return None,
                None => None,
            };
        }

        (!segments.is_empty()).then_some(Self { segments })
    }

    fn specificity(&self) -> usize {
        self.segments
            .iter()
            .flat_map(|segment| segment.tokens.iter())
            .map(|token| match token {
                InspectorSelectorToken::Id(_) => 100,
                InspectorSelectorToken::Class(_)
                | InspectorSelectorToken::State(_)
                | InspectorSelectorToken::Host => 10,
                InspectorSelectorToken::Type(_) => 1,
            })
            .sum()
    }

    fn matches_path(&self, path: &[StyleMatchNode<'_>]) -> bool {
        if path.is_empty() || self.segments.is_empty() {
            return false;
        }

        let mut path_index = path.len() - 1;
        let mut selector_index = self.segments.len() - 1;

        if !selector_segment_matches(&self.segments[selector_index], &path[path_index]) {
            return false;
        }

        while selector_index > 0 {
            let combinator = self.segments[selector_index].combinator;
            selector_index -= 1;
            match combinator {
                Some(InspectorSelectorCombinator::Child) => {
                    if path_index == 0 {
                        return false;
                    }
                    path_index -= 1;
                    if !selector_segment_matches(&self.segments[selector_index], &path[path_index])
                    {
                        return false;
                    }
                }
                Some(InspectorSelectorCombinator::Descendant) => {
                    let mut matched = None;
                    let mut candidate = path_index;
                    while candidate > 0 {
                        candidate -= 1;
                        if selector_segment_matches(
                            &self.segments[selector_index],
                            &path[candidate],
                        ) {
                            matched = Some(candidate);
                            break;
                        }
                    }
                    let Some(found) = matched else {
                        return false;
                    };
                    path_index = found;
                }
                None => return false,
            }
        }

        true
    }
}

fn selector_segment_matches(segment: &InspectorSelectorSegment, node: &StyleMatchNode<'_>) -> bool {
    segment.tokens.iter().all(|token| match token {
        InspectorSelectorToken::Type(component) => node.component == component,
        InspectorSelectorToken::Class(class_name) => {
            node.classes.iter().any(|class| class == class_name)
        }
        InspectorSelectorToken::Id(control_id) => node.control_id == Some(control_id.as_str()),
        InspectorSelectorToken::State(state) => node.states.iter().any(|value| value == state),
        InspectorSelectorToken::Host => node.is_host,
    })
}

fn parse_selector_tokens(input: &str) -> Option<Vec<InspectorSelectorToken>> {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;
    let mut tokens = Vec::new();

    while index < chars.len() {
        let prefix = chars[index];
        let start = if matches!(prefix, '.' | '#' | ':') {
            index + 1
        } else {
            index
        };
        let mut end = start;
        while end < chars.len() && !matches!(chars[end], '.' | '#' | ':') {
            end += 1;
        }
        let value: String = chars[start..end].iter().collect();
        if value.is_empty() {
            return None;
        }
        let token = match prefix {
            '.' => InspectorSelectorToken::Class(value),
            '#' => InspectorSelectorToken::Id(value),
            ':' if value == "host" => InspectorSelectorToken::Host,
            ':' => InspectorSelectorToken::State(value),
            _ => InspectorSelectorToken::Type(value),
        };
        tokens.push(token);
        index = end;
    }

    (!tokens.is_empty()).then_some(tokens)
}

fn skip_selector_whitespace(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    let mut skipped = false;
    while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
        skipped = true;
        let _ = chars.next();
    }
    skipped
}

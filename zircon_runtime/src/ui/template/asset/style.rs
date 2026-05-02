use zircon_runtime_interface::ui::template::{
    UiSelector, UiSelectorCombinator, UiSelectorSegment, UiSelectorToken,
};

#[derive(Clone, Copy, Debug)]
pub struct UiSelectorMatchNode<'a> {
    pub component: &'a str,
    pub control_id: Option<&'a str>,
    pub classes: &'a [String],
    pub is_host: bool,
    pub states: &'a [&'a str],
}

pub(super) trait UiRuntimeSelectorMatchExt {
    fn matches_path(&self, path: &[UiSelectorMatchNode<'_>]) -> bool;
}

impl UiRuntimeSelectorMatchExt for UiSelector {
    fn matches_path(&self, path: &[UiSelectorMatchNode<'_>]) -> bool {
        if path.is_empty() || self.segments.is_empty() {
            return false;
        }

        let mut path_index = path.len() - 1;
        let mut selector_index = self.segments.len() - 1;

        if !matches_segment(&self.segments[selector_index], &path[path_index]) {
            return false;
        }

        while selector_index > 0 {
            let combinator = self.segments[selector_index].combinator;
            selector_index -= 1;
            match combinator {
                Some(UiSelectorCombinator::Child) => {
                    if path_index == 0 {
                        return false;
                    }
                    path_index -= 1;
                    if !matches_segment(&self.segments[selector_index], &path[path_index]) {
                        return false;
                    }
                }
                Some(UiSelectorCombinator::Descendant) => {
                    let mut matched = None;
                    let mut candidate = path_index;
                    while candidate > 0 {
                        candidate -= 1;
                        if matches_segment(&self.segments[selector_index], &path[candidate]) {
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

fn matches_segment(segment: &UiSelectorSegment, node: &UiSelectorMatchNode<'_>) -> bool {
    segment.tokens.iter().all(|token| match token {
        UiSelectorToken::Type(component) => node.component == component.as_str(),
        UiSelectorToken::Class(class_name) => node
            .classes
            .iter()
            .any(|class| class.as_str() == class_name.as_str()),
        UiSelectorToken::Id(control_id) => node.control_id == Some(control_id.as_str()),
        UiSelectorToken::State(state) => node.states.iter().any(|value| *value == state.as_str()),
        UiSelectorToken::Part(_) => false,
        UiSelectorToken::Host => node.is_host,
    })
}

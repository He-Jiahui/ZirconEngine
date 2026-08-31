use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;

mod action;
#[cfg(test)]
#[path = "showcase_event_inputs/action_key_match_tests.rs"]
mod action_key_match_tests;
mod edit;

pub(super) use action::demo_input_for_showcase_action;
pub(super) use edit::demo_input_for_showcase_edit;

const DEFAULT_VIRTUAL_LIST_VISIBLE_COUNT: i64 = 36;
const DEFAULT_PAGED_LIST_PAGE_SIZE: i64 = 100;

fn action_matches(action_id: &str, needle: &str) -> bool {
    iterator_contains(NormalizedActionKeyBytes::new(action_id), needle.bytes())
}

fn action_matches_binding_suffix(action_id: &str, binding_suffix: &str) -> bool {
    iterator_contains(
        NormalizedActionKeyBytes::new(action_id),
        NormalizedSegmentBytes::new(binding_suffix),
    )
}

fn iterator_contains(
    mut haystack: impl Iterator<Item = u8> + Clone,
    needle: impl Iterator<Item = u8> + Clone,
) -> bool {
    let mut needle_tail = needle;
    let Some(first_expected) = needle_tail.next() else {
        return true;
    };
    while let Some(actual) = haystack.next() {
        if actual != first_expected {
            continue;
        }
        let mut candidate = haystack.clone();
        let mut expected = needle_tail.clone();
        if expected.all(|byte| candidate.next() == Some(byte)) {
            return true;
        }
    }
    false
}

#[derive(Clone)]
struct NormalizedActionKeyBytes<'a> {
    remaining: &'a str,
    current_segment: Option<NormalizedSegmentBytes<'a>>,
    emitted_segment: bool,
}

impl<'a> NormalizedActionKeyBytes<'a> {
    fn new(action_id: &'a str) -> Self {
        Self {
            remaining: action_id,
            current_segment: None,
            emitted_segment: false,
        }
    }

    fn take_next_segment(&mut self) -> Option<&'a str> {
        loop {
            if self.remaining.is_empty() {
                return None;
            }
            let delimiter = self
                .remaining
                .as_bytes()
                .iter()
                .position(|byte| matches!(*byte, b'/' | b'.' | b':'));
            let (segment, remaining) = match delimiter {
                Some(index) => (&self.remaining[..index], &self.remaining[index + 1..]),
                None => (self.remaining, ""),
            };
            self.remaining = remaining;
            if !segment.is_empty() {
                return Some(segment);
            }
        }
    }
}

impl Iterator for NormalizedActionKeyBytes<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(segment) = &mut self.current_segment {
                if let Some(byte) = segment.next() {
                    return Some(byte);
                }
                self.current_segment = None;
            }
            let segment = self.take_next_segment()?;
            self.current_segment = Some(NormalizedSegmentBytes::new(segment));
            if self.emitted_segment {
                return Some(b'.');
            }
            self.emitted_segment = true;
        }
    }
}

#[derive(Clone)]
struct NormalizedSegmentBytes<'a> {
    chars: std::str::Chars<'a>,
    emitted_alphanumeric: bool,
    separator_pending: bool,
    pending_byte: Option<u8>,
}

impl<'a> NormalizedSegmentBytes<'a> {
    fn new(segment: &'a str) -> Self {
        Self {
            chars: segment.chars(),
            emitted_alphanumeric: false,
            separator_pending: false,
            pending_byte: None,
        }
    }
}

impl Iterator for NormalizedSegmentBytes<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(byte) = self.pending_byte.take() {
            return Some(byte);
        }
        for ch in self.chars.by_ref() {
            if !ch.is_ascii_alphanumeric() {
                self.separator_pending |= self.emitted_alphanumeric;
                continue;
            }
            let byte = ch.to_ascii_lowercase() as u8;
            let needs_separator =
                self.separator_pending || (ch.is_ascii_uppercase() && self.emitted_alphanumeric);
            self.separator_pending = false;
            self.emitted_alphanumeric = true;
            if needs_separator {
                self.pending_byte = Some(byte);
                return Some(b'_');
            }
            return Some(byte);
        }
        None
    }
}

pub(super) fn select_option(option_id: &str, selected: bool) -> UiComponentShowcaseDemoEventInput {
    UiComponentShowcaseDemoEventInput::SelectOption {
        option_id: option_id.to_string(),
        selected,
    }
}

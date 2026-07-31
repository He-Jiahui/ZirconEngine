use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;

use super::super::{
    DEFAULT_PAGED_LIST_PAGE_SIZE, DEFAULT_VIRTUAL_LIST_VISIBLE_COUNT, action_matches,
};

pub(super) fn demo_list_edit_input(
    action_id: &str,
    value: &str,
) -> Option<UiComponentShowcaseDemoEventInput> {
    if action_matches(action_id, "virtual_list_scrolled") {
        return parse_virtual_list_range(value);
    }
    if action_matches(action_id, "paged_list") {
        return parse_paged_list_request(value);
    }
    None
}

fn parse_virtual_list_range(value: &str) -> Option<UiComponentShowcaseDemoEventInput> {
    let (start, count) = parse_i64_request_pair(
        value,
        &["start", "viewport_start", "requested_start"],
        &["count", "viewport_count", "requested_count"],
        DEFAULT_VIRTUAL_LIST_VISIBLE_COUNT,
    )?;
    Some(UiComponentShowcaseDemoEventInput::SetVisibleRange { start, count })
}

fn parse_paged_list_request(value: &str) -> Option<UiComponentShowcaseDemoEventInput> {
    let (page_index, page_size) = parse_i64_request_pair(
        value,
        &["page", "page_index", "index"],
        &["size", "page_size"],
        DEFAULT_PAGED_LIST_PAGE_SIZE,
    )?;
    Some(UiComponentShowcaseDemoEventInput::SetPage {
        page_index,
        page_size,
    })
}

fn parse_i64_request_pair(
    value: &str,
    first_keys: &[&str],
    second_keys: &[&str],
    default_second: i64,
) -> Option<(i64, i64)> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.contains('=') {
        let mut first = None;
        let mut second = None;
        for part in value.split([',', ';', '&']) {
            let (key, raw_value) = part.split_once('=')?;
            let key = key.trim();
            let parsed_value = raw_value.trim().parse::<i64>().ok()?;
            if first_keys.iter().any(|candidate| key == *candidate) {
                first = Some(parsed_value);
            } else if second_keys.iter().any(|candidate| key == *candidate) {
                second = Some(parsed_value);
            }
        }
        return first.map(|first| (first, second.unwrap_or(default_second)));
    }
    if let Some((first, second)) = value.split_once(',') {
        return Some((first.trim().parse().ok()?, second.trim().parse().ok()?));
    }
    value
        .parse::<i64>()
        .ok()
        .map(|first| (first, default_second))
}

// 查询须由调用方预先转为小写；这里只规范候选文本，不做完整大小写折叠或音符归一化。
pub(super) fn starts_with_lowercase_query(value: &str, lowercase_query: &str) -> bool {
    let value = value.trim_start();
    if lowercase_query.is_empty() {
        return true;
    }
    if value.is_ascii() && lowercase_query.is_ascii() {
        return value
            .as_bytes()
            .get(..lowercase_query.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(lowercase_query.as_bytes()));
    }

    unicode_lowercase_starts_with(value, lowercase_query)
}

fn unicode_lowercase_starts_with(value: &str, lowercase_query: &str) -> bool {
    let mut lowered_value = value.chars().flat_map(char::to_lowercase);
    lowercase_query
        .chars()
        .all(|query_char| lowered_value.next() == Some(query_char))
}

pub(super) fn contains_lowercase_query(value: &str, lowercase_query: &str) -> bool {
    let value = value.trim();
    if lowercase_query.is_empty() {
        return true;
    }
    if value.is_ascii() && lowercase_query.is_ascii() {
        return value
            .as_bytes()
            .windows(lowercase_query.len())
            .any(|candidate| candidate.eq_ignore_ascii_case(lowercase_query.as_bytes()));
    }

    unicode_lowercase_contains(value, lowercase_query)
}

// 每个小写展开后的标量都可成为子串起点，后缀迭代器可复制而无需构造整段小写字符串。
fn unicode_lowercase_contains(value: &str, lowercase_query: &str) -> bool {
    let mut query = lowercase_query.chars();
    let Some(first_query_char) = query.next() else {
        return true;
    };
    let mut lowered = value.chars().flat_map(char::to_lowercase);
    while let Some(candidate) = lowered.next() {
        if candidate != first_query_char {
            continue;
        }
        let mut suffix = lowered.clone();
        if query
            .clone()
            .all(|query_char| suffix.next() == Some(query_char))
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[path = "tests/text_search.rs"]
mod tests;

#[cfg(test)]
#[path = "text_search/tests/streaming_prefix_tests.rs"]
mod streaming_prefix_tests;

#[cfg(test)]
#[path = "text_search/tests/streaming_contains_tests.rs"]
mod streaming_contains_tests;

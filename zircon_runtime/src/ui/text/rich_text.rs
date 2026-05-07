use zircon_runtime_interface::ui::surface::{UiTextRange, UiTextRunKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiTextSourceRun {
    pub kind: UiTextRunKind,
    pub text: String,
    pub source_range: UiTextRange,
}

pub(crate) fn parse_source_runs(text: &str, rich_text: bool) -> Vec<UiTextSourceRun> {
    if !rich_text {
        return vec![UiTextSourceRun {
            kind: UiTextRunKind::Plain,
            text: text.to_string(),
            source_range: UiTextRange {
                start: 0,
                end: text.len(),
            },
        }];
    }

    let mut runs = Vec::new();
    let mut plain_start = 0;
    let mut index = 0;
    while index < text.len() {
        let remaining = &text[index..];
        let marker = rich_marker(remaining);
        let Some((open, close, kind)) = marker else {
            index += next_char_len(remaining);
            continue;
        };

        if let Some(close_offset) = remaining[open.len()..].find(close) {
            if plain_start < index {
                push_run(&mut runs, UiTextRunKind::Plain, text, plain_start, index);
            }
            let content_start = index + open.len();
            let content_end = content_start + close_offset;
            push_run(&mut runs, kind, text, content_start, content_end);
            index = content_end + close.len();
            plain_start = index;
        } else {
            index += open.len();
        }
    }

    if plain_start < text.len() || runs.is_empty() {
        push_run(
            &mut runs,
            UiTextRunKind::Plain,
            text,
            plain_start,
            text.len(),
        );
    }

    runs.retain(|run| !run.text.is_empty());
    runs
}

fn push_run(
    runs: &mut Vec<UiTextSourceRun>,
    kind: UiTextRunKind,
    source: &str,
    start: usize,
    end: usize,
) {
    runs.push(UiTextSourceRun {
        kind,
        text: source[start..end].to_string(),
        source_range: UiTextRange { start, end },
    });
}

fn rich_marker(input: &str) -> Option<(&'static str, &'static str, UiTextRunKind)> {
    if input.starts_with("**") {
        Some(("**", "**", UiTextRunKind::Strong))
    } else if input.starts_with('*') {
        Some(("*", "*", UiTextRunKind::Emphasis))
    } else if input.starts_with('`') {
        Some(("`", "`", UiTextRunKind::Code))
    } else {
        None
    }
}

fn next_char_len(input: &str) -> usize {
    input.chars().next().map(char::len_utf8).unwrap_or(1)
}

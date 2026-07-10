use crate::ui::retained_host::measure_runtime_text_width;

const FILE_NAME_ELLIPSIS: &str = "...";
const MEASURE_EPSILON: f32 = 0.01;

#[derive(Clone, Copy)]
pub(crate) struct RuntimeFileNameCompaction {
    pub(crate) max_width: f32,
    pub(crate) font_size: f32,
    pub(crate) min_prefix_chars: usize,
    pub(crate) min_tail_stem_chars: usize,
    pub(crate) preferred_tail_stem_chars: usize,
}

pub(crate) fn compact_file_like_display_name(
    display_name: &str,
    extension: &str,
    compaction: RuntimeFileNameCompaction,
) -> String {
    let name = display_name.trim();
    if name.is_empty() || fits(name, compaction) {
        return name.to_string();
    }

    if let Some((stem, suffix)) = matching_file_parts(name, extension) {
        let stem_chars = stem.chars().collect::<Vec<_>>();
        if let Some(candidate) = compact_chars(&stem_chars, Some(suffix), compaction) {
            return candidate;
        }
    }

    compact_chars(&name.chars().collect::<Vec<_>>(), None, compaction)
        .unwrap_or_else(|| name.to_string())
}

fn matching_file_parts<'a>(display_name: &'a str, extension: &str) -> Option<(&'a str, &'a str)> {
    let extension = extension.trim().trim_start_matches('.');
    if extension.is_empty() {
        return None;
    }
    let (stem, suffix) = display_name.rsplit_once('.')?;
    suffix
        .eq_ignore_ascii_case(extension)
        .then_some((stem, suffix))
}

fn compact_chars(
    chars: &[char],
    suffix: Option<&str>,
    compaction: RuntimeFileNameCompaction,
) -> Option<String> {
    if chars.len() <= compaction.min_prefix_chars + compaction.min_tail_stem_chars {
        return None;
    }
    let max_tail = compaction
        .preferred_tail_stem_chars
        .max(compaction.min_tail_stem_chars)
        .min(chars.len().saturating_sub(compaction.min_prefix_chars + 1));
    let mut fallback = None;
    for tail_count in (compaction.min_tail_stem_chars..=max_tail).rev() {
        let max_prefix = chars.len().saturating_sub(tail_count + 1);
        for prefix_count in (compaction.min_prefix_chars..=max_prefix).rev() {
            let prefix = chars.iter().take(prefix_count).collect::<String>();
            let tail = chars[chars.len() - tail_count..].iter().collect::<String>();
            let candidate = match suffix {
                Some(suffix) => format!("{prefix}{FILE_NAME_ELLIPSIS}{tail}.{suffix}"),
                None => format!("{prefix}{FILE_NAME_ELLIPSIS}{tail}"),
            };
            fallback = Some(candidate.clone());
            if fits(&candidate, compaction) {
                return Some(candidate);
            }
        }
    }
    fallback
}

fn fits(text: &str, compaction: RuntimeFileNameCompaction) -> bool {
    !compaction.max_width.is_finite()
        || compaction.max_width <= 0.0
        || measure_runtime_text_width(text, compaction.font_size)
            <= compaction.max_width + MEASURE_EPSILON
}

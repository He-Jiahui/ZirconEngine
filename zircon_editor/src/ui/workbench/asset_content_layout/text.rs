use crate::ui::retained_host::measure_runtime_text_width;

const FILE_NAME_ELLIPSIS: &str = "...";
const MEASURE_EPSILON: f32 = 0.01;
const THUMBNAIL_FILE_NAME_MIN_PREFIX_CHARS: usize = 4;
const THUMBNAIL_FILE_NAME_MIN_TAIL_STEM_CHARS: usize = 3;
const THUMBNAIL_FILE_NAME_EXTENSION_TAIL_STEM_CHARS: usize = 4;

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

pub(crate) fn compact_thumbnail_file_name_to_width(
    display_name: &str,
    extension: &str,
    max_width: f32,
    font_size: f32,
) -> String {
    compact_file_like_display_name(
        display_name,
        extension,
        RuntimeFileNameCompaction {
            max_width,
            font_size,
            min_prefix_chars: THUMBNAIL_FILE_NAME_MIN_PREFIX_CHARS,
            min_tail_stem_chars: THUMBNAIL_FILE_NAME_MIN_TAIL_STEM_CHARS,
            preferred_tail_stem_chars: THUMBNAIL_FILE_NAME_EXTENSION_TAIL_STEM_CHARS,
        },
    )
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
    let fallback = compacted_candidate(
        chars,
        compaction.min_prefix_chars,
        compaction.min_tail_stem_chars,
        suffix,
    );
    for tail_count in (compaction.min_tail_stem_chars..=max_tail).rev() {
        let max_prefix = chars.len().saturating_sub(tail_count + 1);
        if let Some(candidate) = largest_fitting_candidate(
            chars,
            suffix,
            tail_count,
            compaction.min_prefix_chars,
            max_prefix,
            compaction,
        ) {
            return Some(candidate);
        }
    }
    Some(fallback)
}

fn largest_fitting_candidate(
    chars: &[char],
    suffix: Option<&str>,
    tail_count: usize,
    min_prefix: usize,
    max_prefix: usize,
    compaction: RuntimeFileNameCompaction,
) -> Option<String> {
    let mut low = min_prefix;
    let mut high = max_prefix;
    let mut best = None;
    while low <= high {
        let prefix_count = low + (high - low) / 2;
        let candidate = compacted_candidate(chars, prefix_count, tail_count, suffix);
        if fits(&candidate, compaction) {
            best = Some(candidate);
            low = prefix_count + 1;
        } else {
            if prefix_count == 0 {
                break;
            }
            high = prefix_count - 1;
        }
    }
    best
}

fn compacted_candidate(
    chars: &[char],
    prefix_count: usize,
    tail_count: usize,
    suffix: Option<&str>,
) -> String {
    let prefix = chars.iter().take(prefix_count).collect::<String>();
    let tail = chars[chars.len() - tail_count..].iter().collect::<String>();
    match suffix {
        Some(suffix) => format!("{prefix}{FILE_NAME_ELLIPSIS}{tail}.{suffix}"),
        None => format!("{prefix}{FILE_NAME_ELLIPSIS}{tail}"),
    }
}

fn fits(text: &str, compaction: RuntimeFileNameCompaction) -> bool {
    !compaction.max_width.is_finite()
        || compaction.max_width <= 0.0
        || measure_runtime_text_width(text, compaction.font_size)
            <= compaction.max_width + MEASURE_EPSILON
}

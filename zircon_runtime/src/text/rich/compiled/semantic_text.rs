use std::sync::Arc;

use crate::text::{InlineObjectRef, RichParseResult, StyledRun};

use super::super::{RichTextParseError, INLINE_OBJECT_REPLACEMENT};

/// 将可见文本中的内联占位符替换成可访问性使用的替代文本，并在构建阶段执行字节预算。
/// 调用方按源顺序传入指向内联运行的索引；函数检查 UTF-8 范围、占位符及扩展后的字节预算。
pub(super) fn semantic_text_for_inline_runs(
    parsed: &RichParseResult,
    inline_run_indices: &[u32],
    max_semantic_text_bytes: usize,
) -> Result<Arc<str>, RichTextParseError> {
    if inline_run_indices.is_empty() {
        admit_semantic_text_bytes(parsed.text.len(), max_semantic_text_bytes)?;
        return Ok(Arc::clone(&parsed.text));
    }

    let mut output = String::with_capacity(parsed.text.len().min(max_semantic_text_bytes));
    let mut cursor = 0;
    let replacement_character = INLINE_OBJECT_REPLACEMENT
        .chars()
        .next()
        .expect("inline object replacement is one character");
    for &run_index in inline_run_indices {
        let run_index = run_index as usize;
        let run = parsed.runs.get(run_index).ok_or(
            RichTextParseError::ArtifactIndexCapacityExceeded {
                index_kind: "inline run",
                actual: run_index,
                max: parsed.runs.len().saturating_sub(1),
            },
        )?;
        let start = run.byte_range.0 as usize;
        let end = run.byte_range.1 as usize;
        if start >= end {
            return Err(invalid_inline_source_range(run, parsed.text.len()));
        }
        let Some(prefix) = (cursor <= start)
            .then(|| parsed.text.get(cursor..start))
            .flatten()
        else {
            return Err(invalid_inline_source_range(run, parsed.text.len()));
        };
        let Some(placeholders) = parsed.text.get(start..end) else {
            return Err(invalid_inline_source_range(run, parsed.text.len()));
        };
        if placeholders
            .chars()
            .any(|character| character != replacement_character)
        {
            return Err(invalid_inline_source_range(run, parsed.text.len()));
        }
        append_semantic_text(&mut output, prefix, max_semantic_text_bytes)?;
        let replacement = inline_semantic_fallback(
            run.inline
                .as_ref()
                .expect("compiled inline index references an inline run"),
        );
        for _ in placeholders.chars() {
            append_semantic_text(&mut output, replacement, max_semantic_text_bytes)?;
        }
        cursor = end;
    }
    let Some(suffix) = parsed.text.get(cursor..) else {
        return Err(RichTextParseError::ArtifactSourceRangeInvalid {
            range_kind: "inline semantic suffix",
            start: u32::try_from(cursor).unwrap_or(u32::MAX),
            end: u32::try_from(parsed.text.len()).unwrap_or(u32::MAX),
            source_bytes: parsed.text.len(),
        });
    };
    append_semantic_text(&mut output, suffix, max_semantic_text_bytes)?;
    Ok(Arc::from(output))
}

fn inline_semantic_fallback(inline: &InlineObjectRef) -> &str {
    // 显式空 alt 也优先于 tooltip；仅图片在 alt 缺失时使用提示，图标无 alt 和 Widget 均为空。
    match inline {
        InlineObjectRef::Image {
            alternative_text: Some(alternative_text),
            ..
        }
        | InlineObjectRef::Icon {
            alternative_text: Some(alternative_text),
            ..
        } => alternative_text,
        InlineObjectRef::Image {
            alternative_text: None,
            tooltip: Some(tooltip),
            ..
        } => tooltip,
        InlineObjectRef::Image {
            alternative_text: None,
            tooltip: None,
            ..
        }
        | InlineObjectRef::Icon {
            alternative_text: None,
            ..
        }
        | InlineObjectRef::Widget { .. } => "",
    }
}

fn append_semantic_text(
    output: &mut String,
    value: &str,
    max_semantic_text_bytes: usize,
) -> Result<(), RichTextParseError> {
    let attempted_bytes = output.len().checked_add(value.len()).unwrap_or(usize::MAX);
    admit_semantic_text_bytes(attempted_bytes, max_semantic_text_bytes)?;
    output.push_str(value);
    Ok(())
}

fn admit_semantic_text_bytes(
    attempted_bytes: usize,
    max_semantic_text_bytes: usize,
) -> Result<(), RichTextParseError> {
    if attempted_bytes > max_semantic_text_bytes {
        return Err(RichTextParseError::SemanticTextByteBudgetExceeded {
            attempted_bytes,
            max_bytes: max_semantic_text_bytes,
        });
    }
    Ok(())
}

fn invalid_inline_source_range(run: &StyledRun, source_bytes: usize) -> RichTextParseError {
    RichTextParseError::ArtifactSourceRangeInvalid {
        range_kind: "inline semantic placeholder",
        start: run.byte_range.0,
        end: run.byte_range.1,
        source_bytes,
    }
}

#[cfg(test)]
#[path = "tests/semantic_text.rs"]
mod tests;

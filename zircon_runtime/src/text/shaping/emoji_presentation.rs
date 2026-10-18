//! 字素簇的 emoji 呈现偏好判定，供段落脚本分析保留原文区间。
//! 这里只查询标量属性与相邻模式；字体覆盖和字形输出由后续流程处理。
//! 调用入口参见父模块的 `script_segment::emoji_presentation_ranges`。

use unicode_properties::{EmojiStatus, UnicodeEmoji};

const TEXT_PRESENTATION_SELECTOR: char = '\u{fe0e}';
const EMOJI_PRESENTATION_SELECTOR: char = '\u{fe0f}';
const COMBINING_ENCLOSING_KEYCAP: char = '\u{20e3}';

/// 决定一个完整字素簇是否采用 emoji 呈现偏好，供脚本分析和后备字体选择共用。
/// 输入须是调用方已分出的字素簇；变体选择符影响呈现偏好，不改变原源字节区间。
///
/// `cluster` 仅在本次调用中借用；空输入返回 `false`，不保存引用。
/// 本函数不验证字素切分或完整 emoji 序列；`true` 也不保证字体覆盖或最终字形。
/// 调用方据此保留原 UTF-8 区间，并将相邻命中区间合并后用于脚本查询。
pub(super) fn cluster_uses_emoji_presentation(cluster: &str) -> bool {
    let mut chars = cluster.chars().peekable();
    while let Some(ch) = chars.next() {
        // 无选择符的键帽相邻模式优先识别，命中后直接给出呈现偏好。
        if is_keycap_base(ch) && chars.peek() == Some(&COMBINING_ENCLOSING_KEYCAP) {
            return true;
        }
        let status = ch.emoji_status();
        if !is_emoji_char(status) {
            continue;
        }
        match chars.peek().copied() {
            Some(TEXT_PRESENTATION_SELECTOR) => {
                // 只消费当前标量后的文本选择符；继续扫描时，后续标量仍可能命中。
                chars.next();
                continue;
            }
            Some(EMOJI_PRESENTATION_SELECTOR) => return true,
            _ => {}
        }
        if has_default_emoji_presentation(status) {
            return true;
        }
    }
    false
}

/// 不带选择符的键帽相邻模式只接受这组 ASCII 基字符。
fn is_keycap_base(ch: char) -> bool {
    matches!(ch, '#' | '*' | '0'..='9')
}

/// 排除非 emoji 状态及仅组件状态，余下标量才进入选择符与默认属性判断。
fn is_emoji_char(status: EmojiStatus) -> bool {
    !matches!(
        status,
        EmojiStatus::NonEmoji | EmojiStatus::NonEmojiButEmojiComponent
    )
}

/// 合并四种带默认 emoji 呈现属性的状态，供未被相邻选择符决定的标量使用。
fn has_default_emoji_presentation(status: EmojiStatus) -> bool {
    matches!(
        status,
        EmojiStatus::EmojiPresentation
            | EmojiStatus::EmojiPresentationAndModifierBase
            | EmojiStatus::EmojiPresentationAndEmojiComponent
            | EmojiStatus::EmojiPresentationAndModifierAndEmojiComponent
    )
}

#[cfg(test)]
#[path = "tests/emoji_presentation.rs"]
mod tests;

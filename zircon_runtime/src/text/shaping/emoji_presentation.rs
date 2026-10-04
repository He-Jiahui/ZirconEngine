use unicode_properties::{EmojiStatus, UnicodeEmoji};

const TEXT_PRESENTATION_SELECTOR: char = '\u{fe0e}';
const EMOJI_PRESENTATION_SELECTOR: char = '\u{fe0f}';
const COMBINING_ENCLOSING_KEYCAP: char = '\u{20e3}';

/// 决定一个完整字素簇是否采用 emoji 呈现偏好，供脚本分析和后备字体选择共用。
/// 输入须是调用方已分出的字素簇；变体选择符影响呈现偏好，不改变原源字节区间。
pub(super) fn cluster_uses_emoji_presentation(cluster: &str) -> bool {
    let mut chars = cluster.chars().peekable();
    while let Some(ch) = chars.next() {
        if is_keycap_base(ch) && chars.peek() == Some(&COMBINING_ENCLOSING_KEYCAP) {
            return true;
        }
        let status = ch.emoji_status();
        if !is_emoji_char(status) {
            continue;
        }
        match chars.peek().copied() {
            Some(TEXT_PRESENTATION_SELECTOR) => {
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

fn is_keycap_base(ch: char) -> bool {
    matches!(ch, '#' | '*' | '0'..='9')
}

fn is_emoji_char(status: EmojiStatus) -> bool {
    !matches!(
        status,
        EmojiStatus::NonEmoji | EmojiStatus::NonEmojiButEmojiComponent
    )
}

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

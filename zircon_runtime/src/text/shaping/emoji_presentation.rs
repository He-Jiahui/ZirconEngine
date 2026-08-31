use unicode_properties::{EmojiStatus, UnicodeEmoji};

const TEXT_PRESENTATION_SELECTOR: char = '\u{fe0e}';
const EMOJI_PRESENTATION_SELECTOR: char = '\u{fe0f}';
const COMBINING_ENCLOSING_KEYCAP: char = '\u{20e3}';

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
mod tests {
    use super::cluster_uses_emoji_presentation;

    #[test]
    fn presentation_follows_unicode_properties_and_variation_selectors() {
        assert!(cluster_uses_emoji_presentation("\u{1f600}"));
        assert!(!cluster_uses_emoji_presentation("\u{2600}"));
        assert!(cluster_uses_emoji_presentation("\u{2600}\u{fe0f}"));
        assert!(!cluster_uses_emoji_presentation("\u{1f600}\u{fe0e}"));
        assert!(!cluster_uses_emoji_presentation("\u{1f02c}"));
        assert!(!cluster_uses_emoji_presentation("A\u{fe0f}"));
    }

    #[test]
    fn keycaps_support_both_standard_selector_forms() {
        assert!(cluster_uses_emoji_presentation("1\u{20e3}"));
        assert!(cluster_uses_emoji_presentation("1\u{fe0f}\u{20e3}"));
    }
}

use unicode_segmentation::UnicodeSegmentation;

const JUSTIFY_EPSILON: f32 = 0.01;

pub(crate) fn justify_line_advances(
    text: &str,
    advances: &[f32],
    natural_width: f32,
    target_width: f32,
) -> Option<Vec<f32>> {
    let extra = target_width - natural_width;
    if extra <= JUSTIFY_EPSILON {
        return None;
    }

    let graphemes = text.graphemes(true).collect::<Vec<_>>();
    if graphemes.len() != advances.len() || graphemes.len() < 2 {
        return None;
    }

    let opportunities = justification_opportunities(&graphemes);
    if opportunities.is_empty() {
        return None;
    }

    let mut adjusted = advances.to_vec();
    let per_opportunity = extra / opportunities.len() as f32;
    let mut assigned = 0.0;
    for (position, index) in opportunities.iter().copied().enumerate() {
        let delta = if position + 1 == opportunities.len() {
            extra - assigned
        } else {
            per_opportunity
        };
        adjusted[index] = (adjusted[index] + delta).max(0.0);
        assigned += delta;
    }

    Some(adjusted)
}

fn justification_opportunities(graphemes: &[&str]) -> Vec<usize> {
    let Some((content_start, content_end)) = content_grapheme_range(graphemes) else {
        return Vec::new();
    };

    let mut opportunities = Vec::new();
    for index in content_start..content_end.saturating_sub(1) {
        if is_word_space(graphemes[index]) {
            opportunities.push(index);
            continue;
        }
        if is_cjk_justifiable_pair(graphemes[index], graphemes[index + 1]) {
            opportunities.push(index);
            continue;
        }
        if is_arabic_kashida_pair(graphemes[index], graphemes[index + 1]) {
            opportunities.push(index);
        }
    }
    opportunities
}

fn content_grapheme_range(graphemes: &[&str]) -> Option<(usize, usize)> {
    let start = graphemes
        .iter()
        .position(|grapheme| !is_word_space(grapheme))?;
    let end = graphemes
        .iter()
        .rposition(|grapheme| !is_word_space(grapheme))?
        + 1;
    Some((start, end))
}

fn is_word_space(grapheme: &str) -> bool {
    matches!(grapheme, " " | "\u{3000}")
}

fn is_cjk_justifiable_pair(left: &str, right: &str) -> bool {
    cjk_char(left).is_some() && cjk_char(right).is_some()
}

fn is_arabic_kashida_pair(left: &str, right: &str) -> bool {
    let Some(left) = single_char(left) else {
        return false;
    };
    let Some(right) = single_char(right) else {
        return false;
    };

    // First kashida slice: distribute extra advance at Arabic joining pairs.
    // Later shaping work can replace this with explicit tatweel glyph insertion.
    is_arabic_left_joining_letter(left) && is_arabic_right_joining_letter(right)
}

fn cjk_char(grapheme: &str) -> Option<char> {
    single_char(grapheme).filter(|ch| {
        matches!(
            *ch as u32,
            0x3040..=0x30FF
                | 0x31F0..=0x31FF
                | 0x3400..=0x4DBF
                | 0x4E00..=0x9FFF
                | 0xAC00..=0xD7AF
                | 0xF900..=0xFAFF
                | 0x20000..=0x2FA1F
        )
    })
}

fn single_char(grapheme: &str) -> Option<char> {
    let mut chars = grapheme.chars();
    let ch = chars.next()?;
    chars.next().is_none().then_some(ch)
}

fn is_arabic_left_joining_letter(ch: char) -> bool {
    is_arabic_letter(ch) && !is_arabic_non_left_joining_letter(ch)
}

fn is_arabic_right_joining_letter(ch: char) -> bool {
    is_arabic_letter(ch)
}

fn is_arabic_letter(ch: char) -> bool {
    matches!(
        ch as u32,
        0x0620..=0x063F
            | 0x0641..=0x064A
            | 0x066E..=0x066F
            | 0x0671..=0x06D3
            | 0x06FA..=0x06FC
            | 0x06FF
            | 0x0750..=0x077F
            | 0x08A0..=0x08C7
            | 0xFB50..=0xFDCF
            | 0xFDF0..=0xFDFF
            | 0xFE70..=0xFEFC
    )
}

fn is_arabic_non_left_joining_letter(ch: char) -> bool {
    matches!(
        ch as u32,
        0x0622..=0x0625
            | 0x0627
            | 0x0629
            | 0x062F..=0x0632
            | 0x0648
            | 0x0671..=0x0673
            | 0x0675..=0x0677
            | 0x0688..=0x0699
            | 0x06C0
            | 0x06C3..=0x06CB
            | 0x06CD
            | 0x06CF
            | 0x06D2..=0x06D3
            | 0x06EE..=0x06EF
    )
}

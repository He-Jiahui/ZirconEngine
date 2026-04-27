use std::collections::BTreeSet;

use zircon_runtime::ui::template::UiAssetDocument;

pub(super) fn unique_style_rule_id(document: &UiAssetDocument, selector: &str) -> String {
    let base = style_rule_id_stem(selector);
    let used_ids = document
        .stylesheets
        .iter()
        .flat_map(|stylesheet| stylesheet.rules.iter())
        .filter_map(|rule| rule.id.as_deref())
        .collect::<BTreeSet<_>>();
    if !used_ids.contains(base.as_str()) {
        return base;
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}_{suffix}");
        if !used_ids.contains(candidate.as_str()) {
            return candidate;
        }
        suffix += 1;
    }
}

fn style_rule_id_stem(selector: &str) -> String {
    let mut stem = String::new();
    let mut previous_was_word = false;
    let mut previous_was_lower_or_digit = false;
    for character in selector.chars() {
        if character.is_ascii_alphanumeric() {
            if character.is_ascii_uppercase() && previous_was_lower_or_digit {
                push_separator(&mut stem);
            }
            stem.push(character.to_ascii_lowercase());
            previous_was_word = true;
            previous_was_lower_or_digit =
                character.is_ascii_lowercase() || character.is_ascii_digit();
        } else {
            if previous_was_word {
                push_separator(&mut stem);
            }
            previous_was_word = false;
            previous_was_lower_or_digit = false;
        }
    }

    while stem.ends_with('_') {
        stem.pop();
    }
    if stem.is_empty() {
        return "style_rule".to_string();
    }
    if stem
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        return format!("rule_{stem}");
    }
    stem
}

fn push_separator(stem: &mut String) {
    if !stem.is_empty() && !stem.ends_with('_') {
        stem.push('_');
    }
}

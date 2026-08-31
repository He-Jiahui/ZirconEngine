#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RetainedOptionSpec {
    pub(crate) raw: String,
    pub(crate) id: String,
    pub(crate) label: String,
    flags: Vec<String>,
}

impl RetainedOptionSpec {
    pub(crate) fn has_flag(&self, expected: &str) -> bool {
        self.flags
            .iter()
            .any(|flag| flag.eq_ignore_ascii_case(expected))
    }

    pub(crate) fn matches_id(&self, expected: &str) -> bool {
        let expected = expected.trim();
        !expected.is_empty()
            && [self.id.as_str(), self.label.as_str(), self.raw.as_str()]
                .into_iter()
                .any(|value| value == expected)
    }

    fn flag_value(&self, expected_key: &str) -> Option<&str> {
        self.flags.iter().find_map(|flag| {
            let (key, value) = flag.split_once('=')?;
            key.trim()
                .eq_ignore_ascii_case(expected_key)
                .then(|| value.trim())
                .filter(|value| !value.is_empty())
        })
    }
}

pub(crate) fn parse_retained_option(raw: &str) -> RetainedOptionSpec {
    let mut parts = raw.splitn(2, '|');
    let id = parts.next().unwrap_or_default().trim();
    let flags = parts
        .next()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|flag| !flag.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut option = RetainedOptionSpec {
        raw: raw.to_string(),
        id: id.to_string(),
        label: id.to_string(),
        flags,
    };
    let label = option
        .flag_value("label")
        .or_else(|| option.flag_value("text"))
        .map(str::to_string);
    if let Some(label) = label {
        option.label = label;
    }
    option
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_option_uses_its_identity_as_the_label() {
        let option = parse_retained_option("surface");

        assert_eq!(option.id, "surface");
        assert_eq!(option.label, "surface");
        assert!(option.matches_id("surface"));
    }

    #[test]
    fn structured_option_keeps_machine_identity_and_display_label() {
        let option = parse_retained_option("post_process|label=Post Process,focused");

        assert_eq!(option.id, "post_process");
        assert_eq!(option.label, "Post Process");
        assert!(option.has_flag("FOCUSED"));
        assert!(option.matches_id("post_process"));
        assert!(option.matches_id("Post Process"));
    }
}

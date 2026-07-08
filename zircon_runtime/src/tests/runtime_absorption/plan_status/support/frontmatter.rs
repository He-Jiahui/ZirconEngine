pub(in crate::tests::runtime_absorption::plan_status) fn frontmatter_status(
    source: &str,
) -> Option<&str> {
    frontmatter_value(source, "status:")
}

pub(in crate::tests::runtime_absorption::plan_status) fn frontmatter_last_refined(
    source: &str,
) -> Option<&str> {
    frontmatter_value(source, "last_refined:")
}

fn frontmatter_value<'a>(source: &'a str, prefix: &str) -> Option<&'a str> {
    let mut lines = source.lines();
    if lines.next() != Some("---") {
        return None;
    }

    for line in lines {
        if line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix(prefix) {
            return Some(value.trim());
        }
    }

    None
}

pub(in crate::tests::runtime_absorption::plan_status) fn markdown_frontmatter_and_body(
    source: &str,
) -> (&str, &str) {
    let source = source
        .strip_prefix("---")
        .expect("markdown document should start with YAML frontmatter");
    let frontmatter_end = source
        .find("\n---")
        .expect("markdown document should close YAML frontmatter");
    let frontmatter = &source[..frontmatter_end];
    let body = &source[frontmatter_end + "\n---".len()..];
    (frontmatter, body)
}

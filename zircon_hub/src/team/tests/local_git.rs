use super::*;

#[test]
fn parse_git_log_authors_counts_and_sorts_recent_authors() {
    let members = parse_git_log_authors(
        "Ada\x1fada@example.com\nLin\x1flin@example.com\nAda\x1fada@example.com\n",
    );

    assert_eq!(members.len(), 2);
    assert_eq!(members[0].name, "Ada");
    assert_eq!(members[0].email, "ada@example.com");
    assert_eq!(members[0].commits, 2);
    assert_eq!(members[1].name, "Lin");
    assert_eq!(members[1].commits, 1);
}

#[test]
fn parse_git_log_authors_skips_invalid_and_empty_lines() {
    let members = parse_git_log_authors("\nmissing separator\n\x1f\nName\x1f\n");

    assert_eq!(members.len(), 1);
    assert_eq!(members[0].name, "Name");
    assert_eq!(members[0].email, "");
    assert_eq!(members[0].commits, 1);
}

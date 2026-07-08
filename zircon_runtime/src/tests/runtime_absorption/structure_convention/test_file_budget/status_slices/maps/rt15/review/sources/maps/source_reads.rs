use super::*;

type SourceExtender = fn(&mut Vec<String>, &str);

#[rustfmt::skip]
pub(in super::super::super) fn read_status_review_foundation_sources() -> String {
    read_foundation_sources(STATUS_REVIEW_FOUNDATION_CHILD, STATUS_REVIEW_FOUNDATION_CHILDREN)
}

#[rustfmt::skip]
pub(in super::super::super) fn read_date_review_foundation_sources() -> String {
    read_foundation_sources(DATE_REVIEW_FOUNDATION_CHILD, DATE_REVIEW_FOUNDATION_CHILDREN)
}

#[rustfmt::skip]
pub(in super::super::super) fn read_status_review_typed_error_sources() -> String {
    read_typed_error_sources(STATUS_REVIEW_TYPED_ERROR_CHILD, STATUS_REVIEW_TYPED_ERROR_CHILDREN)
}

#[rustfmt::skip]
pub(in super::super::super) fn read_date_review_typed_error_sources() -> String {
    read_typed_error_sources(DATE_REVIEW_TYPED_ERROR_CHILD, DATE_REVIEW_TYPED_ERROR_CHILDREN)
}

#[rustfmt::skip]
fn read_foundation_sources(parent: &str, children: &[&str]) -> String {
    read_review_sources(parent, children, child_sources::extend_foundation_expected_slice_sources)
}

#[rustfmt::skip]
fn read_typed_error_sources(parent: &str, children: &[&str]) -> String {
    read_review_sources(parent, children, child_sources::extend_typed_error_status_doc_sources)
}

fn read_review_sources(parent: &str, children: &[&str], extend: SourceExtender) -> String {
    let mut sources = vec![read_runtime_src(parent)];
    for child in children {
        sources.push(read_runtime_src(child));
        extend(&mut sources, child);
    }
    sources.join("\n")
}

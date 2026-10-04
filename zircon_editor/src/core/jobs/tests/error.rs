use super::*;

#[test]
fn cloned_failure_keeps_source_identity_without_text_equality() {
    let failure = JobError::failed(std::io::Error::other("typed source"));
    let clone = failure.clone();

    assert_eq!(failure, clone);
    assert!(failure.downcast_ref::<std::io::Error>().is_some());
}

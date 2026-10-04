use super::*;

#[test]
fn profile_session_basename_is_stable_portable_and_bounded() {
    assert_eq!(profile_session_basename("local"), "local-249f1fb6f3a680e8");
    assert_eq!(
        profile_session_basename("session/with:separators"),
        "session_with_separators-b63ccefbdb6787c6"
    );
    for session_id in ["", ".", "..", "CON", "con.txt", "COM1", "LPT9.log"] {
        let basename = profile_session_basename(session_id);
        assert!(!basename.is_empty(), "session_id={session_id:?}");
        assert!(basename.len() <= PROFILE_SESSION_BASENAME_MAX_BYTES);
        assert_eq!(std::path::Path::new(&basename).components().count(), 1);
    }
}

#[test]
fn profile_session_basename_distinguishes_lossy_and_truncated_ids() {
    let colon = profile_session_basename("a:b");
    let question = profile_session_basename("a?b");
    let already_safe = profile_session_basename("a_b");

    assert_eq!(colon, "a_b-e661911904a01160");
    assert_eq!(question, "a_b-e657a3190497db71");
    assert_ne!(colon, already_safe);
    for basename in [
        colon,
        question,
        already_safe,
        profile_session_basename(&"a".repeat(1_024)),
    ] {
        assert!(basename.len() <= PROFILE_SESSION_BASENAME_MAX_BYTES);
        assert_eq!(std::path::Path::new(&basename).components().count(), 1);
    }
}

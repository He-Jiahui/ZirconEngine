use super::{
    decode_project_session_lock_record, encode_project_session_lock_record,
    project_session_lock_path, ProjectSessionLockRecordV1,
};

#[test]
fn session_lock_record_v1_round_trips_through_the_shared_strict_format() {
    let record = ProjectSessionLockRecordV1::new(913, "913-1723718523000-1", 1_723_718_523_000)
        .expect("fixture instance identity is valid");

    let encoded = encode_project_session_lock_record(&record);

    assert_eq!(
        encoded,
        "version=1\nprocess_id=913\ninstance_id=913-1723718523000-1\nheartbeat_unix_millis=1723718523000\n"
    );
    assert_eq!(
        decode_project_session_lock_record(&encoded).expect("decode shared record"),
        record
    );
}

#[test]
fn session_lock_record_v1_rejects_unknown_or_unsafe_fields() {
    for source in [
        "version=1\nprocess_id=913\ninstance_id=913-1\nheartbeat_unix_millis=2\nlegacy_pid=1\n",
        "version=1\nprocess_id=913\ninstance_id=../unsafe\nheartbeat_unix_millis=2\n",
        "version=2\nprocess_id=913\ninstance_id=913-1\nheartbeat_unix_millis=2\n",
    ] {
        assert!(
            decode_project_session_lock_record(source).is_err(),
            "must reject `{source}`"
        );
    }
}

#[test]
fn session_lock_path_stays_below_the_project_owned_zircon_directory() {
    assert_eq!(
        project_session_lock_path("E:/Projects/My Game"),
        std::path::PathBuf::from("E:/Projects/My Game/.zircon/session.lock")
    );
}

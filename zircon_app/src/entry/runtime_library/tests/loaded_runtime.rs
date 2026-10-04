use super::LoadedRuntime;
use crate::entry::runtime_library::runtime_library_environment_override_request;

#[test]
fn environment_override_load_failure_keeps_the_override_request_provenance() {
    let path = std::env::temp_dir().join(format!(
        "zircon_missing_runtime_override_{}_{}.dll",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos()
    ));
    let request = runtime_library_environment_override_request(&path);
    let error = match LoadedRuntime::load_for_request(&path, request.clone()) {
        Ok(_) => panic!("a nonexistent environment override must fail to load"),
        Err(error) => error,
    };
    let diagnostic = error.to_string();

    assert!(diagnostic.contains(&format!("requested_path={request}")));
    assert!(diagnostic.contains("cause="));
    assert!(diagnostic.contains("recovery=stage the runtime library"));
}

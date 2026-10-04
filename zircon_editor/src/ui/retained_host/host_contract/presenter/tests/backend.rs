use super::*;

#[test]
fn retained_host_defaults_to_gpu_backend() {
    assert_eq!(
        HostPresenterBackend::default_native(),
        HostPresenterBackend::Gpu
    );
    assert_eq!(
        HostPresenterBackend::fallback(),
        HostPresenterBackend::Softbuffer
    );
}

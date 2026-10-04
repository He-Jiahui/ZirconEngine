use super::*;

#[test]
fn zr_vm_real_backend_runtime_lock_recovers_after_poison() {
    let poison_result = std::thread::spawn(|| {
        let _guard = acquire_zr_vm_lock();
        panic!("poison zr_vm real backend runtime lock");
    })
    .join();

    assert!(poison_result.is_err());

    let _guard = acquire_zr_vm_lock();
}

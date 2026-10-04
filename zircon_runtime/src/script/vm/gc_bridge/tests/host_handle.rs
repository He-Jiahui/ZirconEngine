use super::*;

#[test]
fn raw_generation_roundtrip() {
    let handle = HostHandle::from_parts(0x89ab_cdef, 0xfedc_ba98);
    let raw = handle.into_raw();

    assert_eq!(HostHandle::from_raw(raw), handle);
    assert_eq!(HostHandle::from_raw(raw).index(), 0x89ab_cdef);
    assert_eq!(HostHandle::from_raw(raw).generation(), 0xfedc_ba98);
    assert_eq!(raw as i64 as u64, raw);
}

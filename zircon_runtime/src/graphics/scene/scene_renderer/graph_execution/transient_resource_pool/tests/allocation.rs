use super::*;

#[test]
fn allocation_contract_keeps_generation_and_last_use_with_the_native_owner() {
    let source = include_str!("../allocation.rs");

    for owner in [
        "struct TransientTextureAllocation",
        "struct TransientBufferAllocation",
    ] {
        let owner = source
            .split(owner)
            .nth(1)
            .expect("allocation owner must remain declared");
        let fields = owner
            .split('}')
            .next()
            .expect("allocation owner must retain explicit fields");
        for field in [
            "epoch:",
            "key:",
            "desc:",
            "native:",
            "last_used_frame:",
            "byte_size:",
            "last_use_ticket:",
        ] {
            assert!(fields.contains(field), "allocation is missing `{field}`");
        }
    }
}

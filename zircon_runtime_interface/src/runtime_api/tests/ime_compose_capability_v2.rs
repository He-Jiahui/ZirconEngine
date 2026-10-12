use super::*;

#[test]
fn older_or_unadvertised_peers_are_v1_only() {
    let old = ZrRuntimeImeCompositionNegotiation::from_peer(
        7,
        [ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY],
    );
    assert_eq!(
        old.wire_version(),
        ZrRuntimeImeCompositionWireVersion::LegacyV1
    );

    let unadvertised = ZrRuntimeImeCompositionNegotiation::from_peer(8, ["runtime.ime.v1"]);
    assert!(!unadvertised.supports_v2());
    assert_eq!(
        unadvertised.wire_version(),
        ZrRuntimeImeCompositionWireVersion::LegacyV1
    );
}

#[test]
fn exact_schema_and_capability_are_required_before_v2_state_is_emitted() {
    let negotiated = ZrRuntimeImeCompositionNegotiation::from_peer(
        8,
        [
            ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY,
            "runtime.ime.composition.v2.schema=2",
        ],
    );
    assert!(negotiated.supports_v2());
    assert_eq!(
        negotiated.wire_version(),
        ZrRuntimeImeCompositionWireVersion::CompositionV2
    );

    let wrong_schema = ZrRuntimeImeCompositionNegotiation::from_peer(
        8,
        [
            ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY,
            "runtime.ime.composition.v2.schema=3",
        ],
    );
    assert!(!wrong_schema.supports_v2());

    let wrong_schema_before_capability = ZrRuntimeImeCompositionNegotiation::from_peer(
        8,
        [
            "runtime.ime.composition.v2.schema=3",
            ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY,
        ],
    );
    assert!(!wrong_schema_before_capability.supports_v2());
}

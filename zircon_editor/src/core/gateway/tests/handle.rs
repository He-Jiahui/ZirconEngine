use super::{next_generation, GatewayError};

#[test]
fn next_gateway_generation_returns_typed_error_at_u64_max() {
    assert_eq!(
        next_generation(u64::MAX),
        Err(GatewayError::GenerationExhausted)
    );
}

use super::{StandaloneCurveChannel, StandaloneCurveKey};

// These are authoring values, not evaluated samples. Delta no-op/expected-key checks,
// candidate publication and codec reopen all use this same equality. In particular,
// a deliberate -0.0/+0.0 edit must survive instead of being mistaken for a no-op.
// Admission still rejects nonfinite numbers; this does not widen the supported schema.
impl PartialEq for StandaloneCurveKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.input.to_bits() == other.input.to_bits()
            && self.value.to_bits() == other.value.to_bits()
            && self.interpolation == other.interpolation
            && self.tangent_mode == other.tangent_mode
            && optional_bits(self.in_tangent) == optional_bits(other.in_tangent)
            && optional_bits(self.out_tangent) == optional_bits(other.out_tangent)
    }
}

impl PartialEq for StandaloneCurveChannel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.display_name == other.display_name
            && optional_bits(self.default_value) == optional_bits(other.default_value)
            && self.pre_extrapolation == other.pre_extrapolation
            && self.post_extrapolation == other.post_extrapolation
            && self.keys == other.keys
    }
}

fn optional_bits(value: Option<f32>) -> Option<u32> {
    value.map(f32::to_bits)
}

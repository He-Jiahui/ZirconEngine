use std::cell::Cell;
use std::collections::HashMap;

use crate::core::framework::render::ShaderVariantPrewarmSource;

use super::validate_source_once;

#[test]
fn shared_source_wgsl_validation_is_cached_once_per_prewarm_batch() {
    let source = ShaderVariantPrewarmSource::new(
        "res://materials/shared.wgsl",
        "fn main() {}",
        Vec::new(),
        "template-r1",
        "naga-r1",
        "wgpu-r1",
    );
    let mut validation_results = HashMap::new();
    let validation_count = Cell::new(0usize);

    validate_source_once(&mut validation_results, &source, |_| {
        validation_count.set(validation_count.get() + 1);
        Ok(())
    })
    .expect("first validation should pass");
    validate_source_once(&mut validation_results, &source, |_| {
        validation_count.set(validation_count.get() + 1);
        Ok(())
    })
    .expect("cached validation should pass");

    assert_eq!(validation_count.get(), 1);
}

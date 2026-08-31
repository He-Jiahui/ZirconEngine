use crate::ui::binding::{UiBindingConversionRegistry, UiBindingConversionRegistryError};
use zircon_runtime_interface::ui::{
    binding::{
        UiBindingConversionDescriptor, UiBindingConversionId, UiBindingConversionProviderError,
        UiBindingConversionProviderErrorCode, UiBindingConversionProviderGeneration,
        UiBindingConversionSignature,
    },
    component::{UiValue, UiValueKind},
};

#[test]
fn conversion_registry_resolves_exact_typed_signature_and_idempotent_registration() {
    let descriptor = descriptor(1, UiValueKind::Int, UiValueKind::String);
    let mut registry = UiBindingConversionRegistry::new();

    let handle = registry
        .register(descriptor.clone(), int_to_string)
        .unwrap();
    let duplicate = registry
        .register(descriptor.clone(), int_to_string)
        .unwrap();

    assert_eq!(handle, duplicate);
    assert_eq!(registry.revision(), 1);
    assert_eq!(registry.registered_count(), 1);
    assert_eq!(registry.resolve(handle).unwrap(), &descriptor);
    assert_eq!(
        registry
            .resolve_typed(handle, UiValueKind::Int, UiValueKind::String)
            .unwrap(),
        &descriptor
    );
    assert_eq!(
        registry.execute(handle, &UiValue::Int(42)).unwrap(),
        UiValue::String("42".to_string())
    );
}

#[test]
fn conversion_registry_rejects_signature_mismatch_and_generation_conflicts() {
    let mut registry = UiBindingConversionRegistry::new();
    let handle = registry
        .register(
            descriptor(3, UiValueKind::Float, UiValueKind::String),
            float_to_string,
        )
        .unwrap();

    assert!(matches!(
        registry.resolve_typed(handle, UiValueKind::Int, UiValueKind::String),
        Err(UiBindingConversionRegistryError::SignatureMismatch {
            expected_source: UiValueKind::Int,
            actual_source: UiValueKind::Float,
            ..
        })
    ));
    assert!(matches!(
        registry.register(
            descriptor(3, UiValueKind::Int, UiValueKind::String),
            int_to_string
        ),
        Err(UiBindingConversionRegistryError::ProviderGenerationCollision { .. })
    ));
    assert!(matches!(
        registry.register(
            descriptor(2, UiValueKind::Float, UiValueKind::String),
            float_to_string
        ),
        Err(UiBindingConversionRegistryError::ProviderGenerationRegression { .. })
    ));
    assert_eq!(registry.revision(), 1);
}

#[test]
fn conversion_registry_upgrade_and_unload_invalidate_old_handles() {
    let mut registry = UiBindingConversionRegistry::new();
    let first = registry
        .register(
            descriptor(1, UiValueKind::Int, UiValueKind::String),
            int_to_string,
        )
        .unwrap();
    let upgraded_descriptor = descriptor(2, UiValueKind::Float, UiValueKind::String);
    let upgraded = registry
        .register(upgraded_descriptor.clone(), float_to_string)
        .unwrap();

    assert_eq!(first.slot(), upgraded.slot());
    assert_ne!(first.provider_generation(), upgraded.provider_generation());
    assert!(matches!(
        registry.resolve(first),
        Err(UiBindingConversionRegistryError::StaleHandle { handle }) if handle == first
    ));
    assert_eq!(registry.resolve(upgraded).unwrap(), &upgraded_descriptor);
    assert_eq!(registry.revision(), 2);

    let unloaded = registry.unregister(upgraded).unwrap();
    assert_eq!(unloaded, upgraded_descriptor);
    assert_eq!(registry.registered_count(), 0);
    assert_eq!(registry.revision(), 3);
    assert!(matches!(
        registry.resolve(upgraded),
        Err(UiBindingConversionRegistryError::StaleHandle { .. })
    ));
    assert!(matches!(
        registry.unregister(first),
        Err(UiBindingConversionRegistryError::StaleHandle { .. })
    ));
}

#[test]
fn conversion_registry_preserves_provider_errors_and_rejects_wrong_input_or_output_kind() {
    let mut registry = UiBindingConversionRegistry::new();
    let failing = registry
        .register(
            descriptor(1, UiValueKind::Int, UiValueKind::String),
            reject_negative_int,
        )
        .unwrap();
    let wrong_output = registry
        .register(
            UiBindingConversionDescriptor::new(
                UiBindingConversionId::try_new("zircon.number.wrong_output").unwrap(),
                UiBindingConversionProviderGeneration::try_new(1).unwrap(),
                UiBindingConversionSignature::new(UiValueKind::Int, UiValueKind::String),
            ),
            int_to_float,
        )
        .unwrap();

    assert!(matches!(
        registry.execute(failing, &UiValue::Float(1.0)),
        Err(UiBindingConversionRegistryError::InputTypeMismatch {
            expected: UiValueKind::Int,
            actual: UiValueKind::Float,
            ..
        })
    ));
    assert!(matches!(
        registry.execute(failing, &UiValue::Int(-1)),
        Err(UiBindingConversionRegistryError::ProviderFailed {
            error: UiBindingConversionProviderError {
                code: UiBindingConversionProviderErrorCode::InvalidValue,
                ..
            },
            ..
        })
    ));
    assert!(matches!(
        registry.execute(wrong_output, &UiValue::Int(1)),
        Err(UiBindingConversionRegistryError::OutputTypeMismatch {
            expected: UiValueKind::String,
            actual: UiValueKind::Float,
            ..
        })
    ));
}

fn descriptor(
    generation: u64,
    source: UiValueKind,
    destination: UiValueKind,
) -> UiBindingConversionDescriptor {
    UiBindingConversionDescriptor::new(
        UiBindingConversionId::try_new("zircon.number.format").unwrap(),
        UiBindingConversionProviderGeneration::try_new(generation).unwrap(),
        UiBindingConversionSignature::new(source, destination),
    )
}

fn int_to_string(value: &UiValue) -> Result<UiValue, UiBindingConversionProviderError> {
    let UiValue::Int(value) = value else {
        unreachable!("registry validates the source kind before provider execution")
    };
    Ok(UiValue::String(value.to_string()))
}

fn float_to_string(value: &UiValue) -> Result<UiValue, UiBindingConversionProviderError> {
    let UiValue::Float(value) = value else {
        unreachable!("registry validates the source kind before provider execution")
    };
    Ok(UiValue::String(value.to_string()))
}

fn reject_negative_int(value: &UiValue) -> Result<UiValue, UiBindingConversionProviderError> {
    let UiValue::Int(value) = value else {
        unreachable!("registry validates the source kind before provider execution")
    };
    if *value < 0 {
        return Err(UiBindingConversionProviderError::new(
            UiBindingConversionProviderErrorCode::InvalidValue,
            "negative values are not accepted",
        ));
    }
    Ok(UiValue::String(value.to_string()))
}

fn int_to_float(value: &UiValue) -> Result<UiValue, UiBindingConversionProviderError> {
    let UiValue::Int(value) = value else {
        unreachable!("registry validates the source kind before provider execution")
    };
    Ok(UiValue::Float(*value as f64))
}

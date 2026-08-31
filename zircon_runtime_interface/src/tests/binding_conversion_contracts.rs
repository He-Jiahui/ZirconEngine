use crate::ui::{
    binding::{
        UiBindingConversionDescriptor, UiBindingConversionHandle, UiBindingConversionId,
        UiBindingConversionProviderError, UiBindingConversionProviderErrorCode,
        UiBindingConversionProviderGeneration, UiBindingConversionSignature,
        UiBindingConversionSlot,
    },
    component::UiValueKind,
};

#[test]
fn binding_conversion_descriptor_round_trips_typed_signature_handle_and_provider_generation() {
    let descriptor = UiBindingConversionDescriptor::new(
        UiBindingConversionId::try_new("zircon.number.to_string").unwrap(),
        UiBindingConversionProviderGeneration::try_new(7).unwrap(),
        UiBindingConversionSignature::new(UiValueKind::Float, UiValueKind::String),
    );
    let handle = UiBindingConversionHandle::new(
        UiBindingConversionSlot::new(11),
        descriptor.provider_generation,
    );

    let encoded_descriptor = toml::to_string(&descriptor).unwrap();
    let encoded_handle = toml::to_string(&handle).unwrap();
    let decoded_descriptor: UiBindingConversionDescriptor =
        toml::from_str(&encoded_descriptor).unwrap();
    let decoded_handle: UiBindingConversionHandle = toml::from_str(&encoded_handle).unwrap();

    assert_eq!(decoded_descriptor, descriptor);
    assert_eq!(decoded_handle, handle);
    assert_eq!(decoded_handle.slot().get(), 11);
    assert_eq!(decoded_handle.provider_generation().get(), 7);
    assert_eq!(descriptor.signature.source, UiValueKind::Float);
    assert_eq!(descriptor.signature.destination, UiValueKind::String);

    let provider_error = UiBindingConversionProviderError::new(
        UiBindingConversionProviderErrorCode::InvalidValue,
        "non-finite numeric input",
    );
    let encoded_error = toml::to_string(&provider_error).unwrap();
    assert_eq!(
        toml::from_str::<UiBindingConversionProviderError>(&encoded_error).unwrap(),
        provider_error
    );
}

#[test]
fn binding_conversion_identity_and_provider_generation_reject_invalid_contracts() {
    for invalid in ["", "zircon..conversion", "zircon/conversion"] {
        assert!(
            UiBindingConversionId::try_new(invalid).is_err(),
            "{invalid}"
        );
    }
    assert!(UiBindingConversionProviderGeneration::try_new(0).is_err());

    let oversized = "x".repeat(257);
    assert!(UiBindingConversionId::try_new(oversized).is_err());
}

use zircon_runtime::core::framework::sound::SoundError;

use crate::automation::target::ensure_automation_execution_available;

#[test]
fn active_kira_automation_is_typed_m5_unsupported_instead_of_metadata_only_success() {
    let error = ensure_automation_execution_available(true).unwrap_err();

    assert!(matches!(error, SoundError::UnsupportedAdvancedFeature(_)));
    assert!(error.to_string().contains("Sound M5"));
    assert!(ensure_automation_execution_available(false).is_ok());
}

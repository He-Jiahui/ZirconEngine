// 活跃 Kira 下的自动化执行仍受阶段能力门槛约束；测试只确认明确拒绝而非静默接受。
use zircon_runtime::core::framework::sound::SoundError;

use crate::automation::target::ensure_automation_execution_available;

#[test]
fn active_kira_automation_is_typed_m5_unsupported_instead_of_metadata_only_success() {
    let error = ensure_automation_execution_available(true).unwrap_err();

    assert!(matches!(error, SoundError::UnsupportedAdvancedFeature(_)));
    assert!(error.to_string().contains("Sound M5"));
    assert!(ensure_automation_execution_available(false).is_ok());
}

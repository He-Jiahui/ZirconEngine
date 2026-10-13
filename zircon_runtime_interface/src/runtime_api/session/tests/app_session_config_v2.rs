use super::*;

/// 为配置契约测试创建有效的窗口代数和候选矩形输入。
fn configuration() -> ZrRuntimeAppSessionConfigurationV2 {
    ZrRuntimeAppSessionConfigurationV2::ime_composition(
        17,
        ZrRuntimeImeCandidateRectV2::window_relative(0, 24, 0, 18),
    )
}

#[test]
/// 验证描述符完整尺寸和零宽、非零高的窗口坐标矩形。
fn app_session_v2_configuration_has_bounded_candidate_rect_topology() {
    let configuration = configuration();
    assert_eq!(
        configuration.size_bytes,
        size_of::<ZrRuntimeAppSessionConfigurationV2>()
    );
    configuration
        .validate()
        .expect("bounded AppSession V2 config");
    assert_eq!(configuration.candidate_rect.coordinate_space, 1);
    assert_eq!(configuration.candidate_rect.extent_width, 0);
}

#[test]
/// 分别验证尺寸、必需能力和坐标空间的本地错误映射。
fn app_session_v2_rejects_unsupported_contract_before_negotiation() {
    let mut malformed = configuration();
    malformed.size_bytes -= 1;
    assert_eq!(
        malformed.validate(),
        Err(ZrRuntimeAppSessionConfigurationError::SizeMismatch)
    );

    let mut unsupported = configuration();
    unsupported.capability_bits &= !ZR_RUNTIME_APP_SESSION_CAPABILITY_CANDIDATE_RECT_V2;
    assert_eq!(
        unsupported.validate(),
        Err(ZrRuntimeAppSessionConfigurationError::MissingRequiredCapability)
    );

    let mut stale_geometry = configuration();
    stale_geometry.candidate_rect.coordinate_space = 2;
    assert_eq!(
        stale_geometry.validate(),
        Err(ZrRuntimeAppSessionConfigurationError::UnsupportedCoordinateSpace)
    );
}

#[test]
/// 验证插入点矩形构造后仍保留给定窗口原点和高度。
fn app_session_v2_window_relative_rect_preserves_ime_popup_anchor_shape() {
    let rect = ZrRuntimeImeCandidateRectV2::window_relative(21, 34, 0, 19);
    assert_eq!(rect.origin_x, 21);
    assert_eq!(rect.origin_y + rect.extent_height as i32, 53);
}

#[test]
/// 验证候选矩形右边缘越过原生上限时被拒绝。
fn app_session_v2_rejects_rectangles_that_overflow_native_edges() {
    let overflowing = ZrRuntimeAppSessionConfigurationV2::ime_composition(
        17,
        ZrRuntimeImeCandidateRectV2::window_relative(i32::MAX, 0, 1, 18),
    );
    assert_eq!(
        overflowing.validate(),
        Err(ZrRuntimeAppSessionConfigurationError::UnrepresentableCandidateRect)
    );
}

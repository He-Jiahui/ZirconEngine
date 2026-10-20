use zr_rhi::PresentMode;

use super::{choose_present_mode, surface_present_mode};

#[test]
fn present_mode_projection_preserves_concrete_fifo_relaxed() {
    assert_eq!(
        surface_present_mode(wgpu::PresentMode::FifoRelaxed),
        PresentMode::FifoRelaxed
    );
}

#[test]
fn auto_present_mode_negotiation_uses_actual_capabilities() {
    use wgpu::PresentMode as W;
    for caps in [
        [W::Fifo, W::Mailbox, W::Immediate],
        [W::Immediate, W::Mailbox, W::Fifo],
    ] {
        assert_eq!(
            choose_present_mode(&caps, PresentMode::AutoNoVsync),
            Some(W::Immediate)
        );
    }
    assert_eq!(
        choose_present_mode(&[W::Fifo, W::Mailbox], PresentMode::AutoNoVsync),
        Some(W::Mailbox)
    );
    assert_eq!(
        choose_present_mode(&[W::Fifo], PresentMode::AutoNoVsync),
        Some(W::Fifo)
    );
    assert_eq!(
        choose_present_mode(&[W::Fifo, W::FifoRelaxed], PresentMode::AutoVsync),
        Some(W::FifoRelaxed)
    );
    assert_eq!(
        choose_present_mode(&[W::Fifo], PresentMode::AutoVsync),
        Some(W::Fifo)
    );
    assert_eq!(
        choose_present_mode(&[W::Mailbox], PresentMode::AutoVsync),
        None
    );
    assert_eq!(choose_present_mode(&[], PresentMode::AutoNoVsync), None);
    assert_eq!(
        choose_present_mode(&[W::AutoNoVsync], PresentMode::AutoNoVsync),
        None
    );
}

#[test]
fn explicit_present_mode_rejects_unavailable_capabilities() {
    use wgpu::PresentMode as W;
    for (request, concrete) in [
        (PresentMode::Immediate, W::Immediate),
        (PresentMode::Fifo, W::Fifo),
        (PresentMode::Mailbox, W::Mailbox),
        (PresentMode::FifoRelaxed, W::FifoRelaxed),
    ] {
        assert_eq!(choose_present_mode(&[concrete], request), Some(concrete));
        assert_eq!(surface_present_mode(concrete), request);
        assert_eq!(choose_present_mode(&[], request), None);
        for other in [W::Immediate, W::Fifo, W::Mailbox, W::FifoRelaxed] {
            if other != concrete {
                assert_eq!(choose_present_mode(&[other], request), None);
            }
        }
    }
}

#[test]
fn native_fifo_variants_project_to_the_neutral_fifo_contract() {
    for mode in [
        wgpu::PresentMode::Fifo,
        wgpu::PresentMode::AutoVsync,
        wgpu::PresentMode::AutoNoVsync,
    ] {
        assert_eq!(surface_present_mode(mode), PresentMode::Fifo);
    }
}

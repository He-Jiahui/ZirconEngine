use zr_rhi::PresentMode;

use super::surface_present_mode;

#[test]
fn native_fifo_variants_project_to_the_neutral_fifo_contract() {
    for mode in [
        wgpu::PresentMode::Fifo,
        wgpu::PresentMode::FifoRelaxed,
        wgpu::PresentMode::AutoVsync,
        wgpu::PresentMode::AutoNoVsync,
    ] {
        assert_eq!(surface_present_mode(mode), PresentMode::Fifo);
    }
}

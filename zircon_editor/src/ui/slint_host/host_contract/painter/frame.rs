pub(in crate::ui::slint_host::host_contract) struct HostRgbaFrame {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
}

impl HostRgbaFrame {
    pub(in crate::ui::slint_host::host_contract) fn empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bytes: Vec::new(),
        }
    }

    pub(in crate::ui::slint_host::host_contract) fn filled(
        width: u32,
        height: u32,
        color: [u8; 4],
    ) -> Self {
        let mut bytes = vec![0; width as usize * height as usize * 4];
        for pixel in bytes.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
        Self {
            width,
            height,
            bytes,
        }
    }

    pub(in crate::ui::slint_host::host_contract) fn width(&self) -> u32 {
        self.width
    }

    pub(in crate::ui::slint_host::host_contract) fn height(&self) -> u32 {
        self.height
    }

    pub(in crate::ui::slint_host::host_contract) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(in crate::ui::slint_host::host_contract) fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    pub(in crate::ui::slint_host::host_contract) fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

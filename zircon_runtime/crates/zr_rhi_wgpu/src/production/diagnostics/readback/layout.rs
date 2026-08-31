use zr_rhi::TextureCopyRegion;

const TEXTURE_ROW_ALIGNMENT: u64 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64;

/// Row-padded staging metadata for a single color texture subresource copy.
/// The public delivery keeps only tightly packed source rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DiagnosticTextureReadbackLayout {
    unpadded_bytes_per_row: u32,
    padded_bytes_per_row: u32,
    staging_byte_len: u64,
    height: u32,
}

impl DiagnosticTextureReadbackLayout {
    pub(crate) fn new(unpadded_bytes_per_row: u64, height: u32) -> Option<Self> {
        if unpadded_bytes_per_row == 0 || height == 0 {
            return None;
        }
        let padded_bytes_per_row = unpadded_bytes_per_row
            .checked_add(TEXTURE_ROW_ALIGNMENT.checked_sub(1)?)?
            .checked_div(TEXTURE_ROW_ALIGNMENT)?
            .checked_mul(TEXTURE_ROW_ALIGNMENT)?;
        let staging_byte_len = padded_bytes_per_row.checked_mul(u64::from(height))?;
        Some(Self {
            unpadded_bytes_per_row: u32::try_from(unpadded_bytes_per_row).ok()?,
            padded_bytes_per_row: u32::try_from(padded_bytes_per_row).ok()?,
            staging_byte_len,
            height,
        })
    }

    pub(crate) const fn padded_bytes_per_row(self) -> u32 {
        self.padded_bytes_per_row
    }

    pub(crate) const fn staging_byte_len(self) -> u64 {
        self.staging_byte_len
    }

    pub(crate) const fn height(self) -> u32 {
        self.height
    }

    pub(crate) fn payload_byte_len(self) -> Option<usize> {
        (self.unpadded_bytes_per_row as usize).checked_mul(self.height as usize)
    }

    fn append_unpacked(self, mapped: &[u8], payload: &mut Vec<u8>) -> Option<()> {
        let staging_byte_len = usize::try_from(self.staging_byte_len).ok()?;
        if mapped.len() < staging_byte_len {
            return None;
        }
        let row_bytes = self.unpadded_bytes_per_row as usize;
        let padded_row_bytes = self.padded_bytes_per_row as usize;
        payload.reserve(self.payload_byte_len()?);
        for row in 0..self.height as usize {
            let start = row.checked_mul(padded_row_bytes)?;
            let end = start.checked_add(row_bytes)?;
            payload.extend_from_slice(mapped.get(start..end)?);
        }
        Some(())
    }

    pub(crate) fn unpack(self, mapped: &[u8]) -> Option<Vec<u8>> {
        let mut payload = Vec::with_capacity(self.payload_byte_len()?);
        self.append_unpacked(mapped, &mut payload)?;
        Some(payload)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DiagnosticTextureReadbackSubresource {
    region: TextureCopyRegion,
    layout: DiagnosticTextureReadbackLayout,
    staging_offset: u64,
}

impl DiagnosticTextureReadbackSubresource {
    pub(crate) const fn region(self) -> TextureCopyRegion {
        self.region
    }

    pub(crate) const fn layout(self) -> DiagnosticTextureReadbackLayout {
        self.layout
    }

    pub(crate) const fn staging_offset(self) -> u64 {
        self.staging_offset
    }
}

pub(crate) struct DiagnosticTextureMipChainReadbackLayout {
    subresources: Vec<DiagnosticTextureReadbackSubresource>,
    staging_byte_len: u64,
    payload_byte_len: usize,
}

impl DiagnosticTextureMipChainReadbackLayout {
    pub(crate) fn new(
        subresources: impl IntoIterator<Item = (TextureCopyRegion, DiagnosticTextureReadbackLayout)>,
    ) -> Option<Self> {
        let mut packed = Vec::new();
        let mut staging_byte_len = 0_u64;
        let mut payload_byte_len = 0_usize;
        for (region, layout) in subresources {
            packed.push(DiagnosticTextureReadbackSubresource {
                region,
                layout,
                staging_offset: staging_byte_len,
            });
            staging_byte_len = staging_byte_len.checked_add(layout.staging_byte_len())?;
            payload_byte_len = payload_byte_len.checked_add(layout.payload_byte_len()?)?;
        }
        if packed.is_empty() {
            return None;
        }
        Some(Self {
            subresources: packed,
            staging_byte_len,
            payload_byte_len,
        })
    }

    pub(crate) fn subresources(&self) -> &[DiagnosticTextureReadbackSubresource] {
        &self.subresources
    }

    pub(crate) const fn staging_byte_len(&self) -> u64 {
        self.staging_byte_len
    }

    pub(crate) fn unpack(&self, mapped: &[u8]) -> Option<Vec<u8>> {
        let staging_byte_len = usize::try_from(self.staging_byte_len).ok()?;
        if mapped.len() < staging_byte_len {
            return None;
        }
        let mut payload = Vec::with_capacity(self.payload_byte_len);
        for subresource in &self.subresources {
            let start = usize::try_from(subresource.staging_offset()).ok()?;
            let byte_len = usize::try_from(subresource.layout().staging_byte_len()).ok()?;
            let end = start.checked_add(byte_len)?;
            subresource
                .layout()
                .append_unpacked(mapped.get(start..end)?, &mut payload)?;
        }
        Some(payload)
    }
}

pub(super) const fn align_up(value: u64, alignment: u64) -> Option<u64> {
    if alignment == 0 {
        return None;
    }
    match value.checked_add(alignment - 1) {
        Some(value) => Some(value / alignment * alignment),
        None => None,
    }
}

pub(super) const fn texture_row_alignment() -> u64 {
    TEXTURE_ROW_ALIGNMENT
}

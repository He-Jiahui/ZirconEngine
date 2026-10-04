use super::RenderViewportHandle;

const VIEWPORT_RESOURCE_KEY_PREFIX: &str = "viewport:";

/// Backend-neutral identity for a GPU-resident viewport presentation product.
///
/// The descriptor intentionally contains no native texture handle. The render backend retains
/// that owner behind the resource key; consumers can safely carry this value across the
/// runtime/editor boundary and fall back to an explicit CPU capture when no matching presenter
/// product is available.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderViewportProduct {
    resource_key: String,
    width: u32,
    height: u32,
    generation: u64,
}

impl RenderViewportProduct {
    pub fn new(viewport: RenderViewportHandle, width: u32, height: u32, generation: u64) -> Self {
        Self {
            resource_key: viewport_resource_key(viewport.raw(), generation),
            width,
            height,
            generation,
        }
    }

    pub fn resource_key(&self) -> &str {
        &self.resource_key
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    // TODO: [CR-RENDERROOT-0001] 核定视口句柄 0 的合法性：拾取请求拒绝 0，
    // 但此处可将 viewport:0 的产品判为有效；补齐跨产品契约和边界测试。
    pub const fn is_valid(&self) -> bool {
        !self.resource_key.is_empty() && self.width != 0 && self.height != 0 && self.generation != 0
    }
}

fn viewport_resource_key(viewport: u64, generation: u64) -> String {
    let (viewport_digits, viewport_start) = decimal_digits(viewport);
    let (generation_digits, generation_start) = decimal_digits(generation);
    let viewport_len = viewport_digits.len() - viewport_start;
    let generation_len = generation_digits.len() - generation_start;
    let capacity = VIEWPORT_RESOURCE_KEY_PREFIX.len() + viewport_len + 1 + generation_len;
    let mut key = String::with_capacity(capacity);
    key.push_str(VIEWPORT_RESOURCE_KEY_PREFIX);
    push_ascii_digits(&mut key, &viewport_digits[viewport_start..]);
    key.push(':');
    push_ascii_digits(&mut key, &generation_digits[generation_start..]);
    key
}

fn decimal_digits(mut value: u64) -> ([u8; 20], usize) {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    (digits, start)
}

fn push_ascii_digits(output: &mut String, digits: &[u8]) {
    for digit in digits {
        output.push(char::from(*digit));
    }
}

#[cfg(test)]
#[path = "tests/viewport_product.rs"]
mod tests;

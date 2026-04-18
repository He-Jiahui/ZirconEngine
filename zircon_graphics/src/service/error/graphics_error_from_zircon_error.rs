use zircon_core::ZirconError;

use crate::types::GraphicsError;

impl From<ZirconError> for GraphicsError {
    fn from(value: ZirconError) -> Self {
        Self::ThreadBootstrap(value.to_string())
    }
}

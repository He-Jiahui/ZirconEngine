use zircon_runtime::core::framework::animation::AnimationAssetError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationEditorBinaryKindMismatch {
    expected: &'static str,
    actual: &'static str,
}

impl AnimationEditorBinaryKindMismatch {
    pub(crate) const fn new(expected: &'static str, actual: &'static str) -> Self {
        Self { expected, actual }
    }

    pub const fn expected(self) -> &'static str {
        self.expected
    }

    pub const fn actual(self) -> &'static str {
        self.actual
    }

    pub const fn code(self) -> &'static str {
        "ZR-ANIM-LOAD-001"
    }
}

#[derive(Clone, Debug)]
pub struct AnimationEditorSessionError {
    message: String,
    binary_kind_mismatch: Option<AnimationEditorBinaryKindMismatch>,
}

impl AnimationEditorSessionError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            binary_kind_mismatch: None,
        }
    }

    pub(crate) fn from_animation_asset_error(error: AnimationAssetError) -> Self {
        match error.binary_kind_mismatch() {
            Some((expected, actual)) => Self {
                message: format!(
                    "[ZR-ANIM-LOAD-001] animation binary kind mismatch: expected {expected}, found {actual}"
                ),
                binary_kind_mismatch: Some(AnimationEditorBinaryKindMismatch::new(
                    expected, actual,
                )),
            },
            None => Self::new(error.to_string()),
        }
    }

    pub fn binary_kind_mismatch(&self) -> Option<AnimationEditorBinaryKindMismatch> {
        self.binary_kind_mismatch
    }
}

impl std::fmt::Display for AnimationEditorSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AnimationEditorSessionError {}

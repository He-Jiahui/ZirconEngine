#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteAtlasBuildConfig {
    pub output_stem: String,
    pub padding: (u32, u32),
    pub initial_size: (u32, u32),
    pub max_size: (u32, u32),
}

impl SpriteAtlasBuildConfig {
    pub(super) fn validate(&self) -> Result<(), String> {
        if self.output_stem.trim().is_empty() {
            return Err("output_stem must not be empty".to_string());
        }
        if self.output_stem.trim() != self.output_stem
            || self.output_stem == "."
            || self.output_stem == ".."
            || !is_safe_output_stem(&self.output_stem)
            || self.output_stem.ends_with('.')
            || is_windows_reserved_stem(&self.output_stem)
        {
            return Err("output_stem must be a single safe file stem".to_string());
        }
        if self.initial_size.0 == 0
            || self.initial_size.1 == 0
            || self.max_size.0 == 0
            || self.max_size.1 == 0
        {
            return Err("initial_size and max_size must be non-zero".to_string());
        }
        if self.initial_size.0 > self.max_size.0 || self.initial_size.1 > self.max_size.1 {
            return Err("initial_size must fit inside max_size".to_string());
        }
        Ok(())
    }
}

fn is_safe_output_stem(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn is_windows_reserved_stem(value: &str) -> bool {
    let stem = value
        .split_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(value)
        .to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

impl Default for SpriteAtlasBuildConfig {
    fn default() -> Self {
        Self {
            output_stem: "editor-atlas".to_string(),
            padding: (1, 1),
            initial_size: (256, 256),
            max_size: (2048, 2048),
        }
    }
}

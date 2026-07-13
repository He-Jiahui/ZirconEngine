use std::ffi::OsString;
use std::path::PathBuf;

use zircon_runtime::graphics::text::font_sdf_build_tool::{
    FontSdfBakeMode, FontSdfBakeRequest, FontSdfGlyphSelection,
};

pub(super) struct FontSdfCliArgs {
    pub(super) font: PathBuf,
    pub(super) cache_root: PathBuf,
    pub(super) request: FontSdfBakeRequest,
}

impl FontSdfCliArgs {
    pub(super) fn parse(
        values: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, FontSdfCliError> {
        let mut values = values.into_iter();
        let mut font = None;
        let mut cache_root = None;
        let mut asset_guid = None;
        let mut face_index = 0;
        let mut mode = FontSdfBakeMode::Sdf;
        let mut page_size = 1024;
        let mut bake_em_px = 48;
        let mut spread_px_milli = 8_000;
        let mut variation_hash = *blake3::hash(&[]).as_bytes();
        let mut codepoints = Vec::new();
        let mut all_cmap = false;

        while let Some(flag) = values.next() {
            let flag = flag
                .into_string()
                .map_err(|_| FontSdfCliError("argument flag is not UTF-8".to_string()))?;
            match flag.as_str() {
                "--font" => font = Some(PathBuf::from(next(&mut values, &flag)?)),
                "--cache-root" => cache_root = Some(PathBuf::from(next(&mut values, &flag)?)),
                "--asset-guid" => asset_guid = Some(text(next(&mut values, &flag)?, &flag)?),
                "--face-index" => face_index = number(next(&mut values, &flag)?, &flag)?,
                "--mode" => mode = parse_mode(next(&mut values, &flag)?)?,
                "--page-size" => page_size = number(next(&mut values, &flag)?, &flag)?,
                "--bake-em-px" => bake_em_px = number(next(&mut values, &flag)?, &flag)?,
                "--spread-px-milli" => spread_px_milli = number(next(&mut values, &flag)?, &flag)?,
                "--variation-hash" => variation_hash = hash(next(&mut values, &flag)?, &flag)?,
                "--codepoint" => codepoints.push(codepoint(next(&mut values, &flag)?)?),
                "--codepoint-range" => {
                    codepoints.extend(codepoint_range(next(&mut values, &flag)?)?)
                }
                "--all-cmap" => all_cmap = true,
                _ => return Err(FontSdfCliError(format!("unknown argument {flag}"))),
            }
        }
        codepoints.sort_unstable();
        codepoints.dedup();
        if all_cmap == !codepoints.is_empty() {
            return Err(FontSdfCliError(
                "select exactly one of --all-cmap or repeated --codepoint".to_string(),
            ));
        }
        Ok(Self {
            font: font.ok_or_else(|| FontSdfCliError("missing --font".to_string()))?,
            cache_root: cache_root
                .ok_or_else(|| FontSdfCliError("missing --cache-root".to_string()))?,
            request: FontSdfBakeRequest {
                asset_guid: asset_guid
                    .ok_or_else(|| FontSdfCliError("missing --asset-guid".to_string()))?,
                face_index,
                variation_hash,
                mode,
                page_size,
                bake_em_px,
                spread_px_milli,
                selection: if all_cmap {
                    FontSdfGlyphSelection::AllCmap
                } else {
                    FontSdfGlyphSelection::Codepoints(codepoints)
                },
            },
        })
    }
}

#[derive(Debug)]
pub(super) struct FontSdfCliError(String);

impl std::fmt::Display for FontSdfCliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for FontSdfCliError {}

fn next(
    values: &mut impl Iterator<Item = OsString>,
    flag: &str,
) -> Result<OsString, FontSdfCliError> {
    values
        .next()
        .ok_or_else(|| FontSdfCliError(format!("{flag} requires a value")))
}

fn text(value: OsString, flag: &str) -> Result<String, FontSdfCliError> {
    value
        .into_string()
        .map_err(|_| FontSdfCliError(format!("{flag} value is not UTF-8")))
}

fn number(value: OsString, flag: &str) -> Result<u32, FontSdfCliError> {
    text(value, flag)?
        .parse()
        .map_err(|_| FontSdfCliError(format!("{flag} requires a u32 value")))
}

fn parse_mode(value: OsString) -> Result<FontSdfBakeMode, FontSdfCliError> {
    match text(value, "--mode")?.as_str() {
        "sdf" => Ok(FontSdfBakeMode::Sdf),
        "msdf" => Ok(FontSdfBakeMode::Msdf),
        "mtsdf" => Ok(FontSdfBakeMode::Mtsdf),
        mode => Err(FontSdfCliError(format!(
            "unsupported --mode {mode}; expected sdf, msdf, or mtsdf"
        ))),
    }
}

fn codepoint(value: OsString) -> Result<u32, FontSdfCliError> {
    let value = text(value, "--codepoint")?;
    let digits = value
        .strip_prefix("U+")
        .or_else(|| value.strip_prefix("u+"))
        .ok_or_else(|| FontSdfCliError(format!("invalid codepoint {value}")))?;
    let codepoint = u32::from_str_radix(digits, 16)
        .map_err(|_| FontSdfCliError(format!("invalid codepoint {value}")))?;
    char::from_u32(codepoint)
        .map(|_| codepoint)
        .ok_or_else(|| FontSdfCliError(format!("invalid Unicode scalar {value}")))
}

fn codepoint_range(value: OsString) -> Result<Vec<u32>, FontSdfCliError> {
    let value = text(value, "--codepoint-range")?;
    let (start, end) = value
        .split_once('-')
        .ok_or_else(|| FontSdfCliError(format!("invalid codepoint range {value}")))?;
    let start = codepoint(OsString::from(start))?;
    let end = codepoint(OsString::from(end))?;
    if end < start {
        return Err(FontSdfCliError(format!(
            "codepoint range is reversed: {value}"
        )));
    }
    let mut codepoints = Vec::with_capacity((end - start + 1) as usize);
    for scalar in start..=end {
        if char::from_u32(scalar).is_none() {
            return Err(FontSdfCliError(format!(
                "codepoint range contains a non-scalar value: {value}"
            )));
        }
        codepoints.push(scalar);
    }
    Ok(codepoints)
}

fn hash(value: OsString, flag: &str) -> Result<[u8; 32], FontSdfCliError> {
    let value = text(value, flag)?;
    if value.len() != 64 {
        return Err(FontSdfCliError(format!(
            "{flag} requires 64 hexadecimal digits"
        )));
    }
    let mut decoded = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let pair = std::str::from_utf8(pair)
            .map_err(|_| FontSdfCliError(format!("{flag} must be hexadecimal")))?;
        decoded[index] = u8::from_str_radix(pair, 16)
            .map_err(|_| FontSdfCliError(format!("{flag} must be hexadecimal")))?;
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests;

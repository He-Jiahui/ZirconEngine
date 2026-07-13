use super::AssetRefError;

pub(super) fn validate_sub_path(sub: &str) -> Result<(), AssetRefError> {
    if sub.is_empty() {
        return Err(AssetRefError::EmptySubPath);
    }
    if sub.contains('#') {
        return Err(AssetRefError::FragmentDelimiterInSubPath);
    }
    if let Some((index, _)) = sub
        .char_indices()
        .find(|(_, character)| character.is_control())
    {
        return Err(AssetRefError::ControlCharacterInSubPath { index });
    }
    Ok(())
}

use std::path::Path;

use crate::asset::pack::{ZrPackDeltaReader, ZrPackError, ZrPackReader};

use super::{
    file_io::{read_pack_file, write_pack_file},
    ZrPackDeltaInstallError, ZrPackDeltaInstallReport,
};

pub(super) fn rebuild_to_staging(
    base_pack: &Path,
    delta_pack: &Path,
    staged_pack: &Path,
) -> Result<ZrPackDeltaInstallReport, ZrPackDeltaInstallError> {
    let base_bytes = read_pack_file(base_pack)?;
    let delta_bytes = read_pack_file(delta_pack)?;
    let base_reader = ZrPackReader::from_bytes(base_bytes)?;
    let delta_reader = ZrPackDeltaReader::from_bytes(delta_bytes)?;
    let rebuilt = delta_reader.apply_to_base(&base_reader)?;

    write_pack_file(staged_pack, &rebuilt.bytes)?;

    let staged_size = u64::try_from(rebuilt.bytes.len()).map_err(|_| ZrPackError::SizeOverflow)?;
    Ok(ZrPackDeltaInstallReport {
        base_pack: base_pack.to_path_buf(),
        delta_pack: delta_pack.to_path_buf(),
        staged_pack: staged_pack.to_path_buf(),
        target_manifest: rebuilt.manifest,
        staged_size,
        delta_apply_verified: true,
    })
}

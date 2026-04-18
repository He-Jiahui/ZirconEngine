use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

pub(super) fn source_mtime_unix_ms(path: &Path) -> Result<u64, std::io::Error> {
    let modified = fs::metadata(path)?.modified()?;
    let duration = modified.duration_since(UNIX_EPOCH).unwrap_or_default();
    Ok(duration.as_millis() as u64)
}

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use serde::Serialize;

use super::{path_string, ProfileExportError, ProfileExportResult};

const JSON_WRITE_BUFFER_BYTES: usize = 64 * 1024;

pub(super) fn write_json<T: Serialize>(
    dir: &Path,
    name: &'static str,
    value: &T,
) -> ProfileExportResult<()> {
    let path = dir.join(name);
    let file = File::create(&path).map_err(|source| ProfileExportError::WriteFile {
        path: path_string(&path),
        source,
    })?;
    write_buffered_json(file, &path, name, value)
}

// JSON 通过固定容量缓冲流式写出并显式 flush；这里区分序列化错误与文件写入错误。
pub(super) fn write_buffered_json<W: Write, T: Serialize>(
    writer: W,
    path: &Path,
    name: &'static str,
    value: &T,
) -> ProfileExportResult<()> {
    let mut writer = BufWriter::with_capacity(JSON_WRITE_BUFFER_BYTES, writer);
    write_pretty_json(&mut writer, path, name, value)?;
    writer
        .flush()
        .map_err(|source| ProfileExportError::WriteFile {
            path: path_string(&path),
            source,
        })
}

pub(super) fn write_pretty_json<W: Write, T: Serialize>(
    writer: &mut W,
    path: &Path,
    name: &'static str,
    value: &T,
) -> ProfileExportResult<()> {
    serde_json::to_writer_pretty(writer, value).map_err(|source| {
        if source.is_io() {
            let kind = source.io_error_kind().unwrap_or(io::ErrorKind::Other);
            ProfileExportError::WriteFile {
                path: path_string(path),
                source: io::Error::new(kind, source),
            }
        } else {
            ProfileExportError::JsonSerialize { file: name, source }
        }
    })
}

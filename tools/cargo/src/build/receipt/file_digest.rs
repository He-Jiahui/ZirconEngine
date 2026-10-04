//! 已有权限文件句柄的流式摘要。
//! 构建捕获和落盘核验都复用缓冲与已打开句柄，统计长度和哈希以绑定实际读到的字节；路径准入由上层负责。

use std::fs::File;
use std::io::Read;

use sha2::{Digest, Sha256};

use super::{canonical::bytes_to_hex, ProductReceiptError};

const OPEN_FILE_HASH_READ_BUFFER_BYTES: usize = 64 * 1024;

/// 多个已打开产物间复用的读取缓冲；每个文件仍创建独立摘要状态。
pub(crate) struct FileDigestBuffer {
    bytes: [u8; OPEN_FILE_HASH_READ_BUFFER_BYTES],
}

impl FileDigestBuffer {
    pub(crate) fn new() -> Self {
        Self {
            bytes: [0_u8; OPEN_FILE_HASH_READ_BUFFER_BYTES],
        }
    }
}

pub(crate) struct FileDigest {
    pub(crate) sha256: String,
    pub(crate) byte_length: u64,
}

pub(crate) struct RawFileDigest {
    pub(crate) sha256: [u8; 32],
    pub(crate) byte_length: u64,
}

// Callers supply an already-authorized file handle; this module never resolves a path again.
pub(crate) fn digest_open_file(mut file: File) -> Result<FileDigest, ProductReceiptError> {
    digest_open_file_handle(&mut file)
}

pub(crate) fn digest_open_file_with_buffer(
    mut file: File,
    buffer: &mut FileDigestBuffer,
) -> Result<FileDigest, ProductReceiptError> {
    digest_open_file_handle_with_buffer(&mut file, buffer)
}

pub(crate) fn digest_open_file_handle_with_buffer(
    file: &mut File,
    buffer: &mut FileDigestBuffer,
) -> Result<FileDigest, ProductReceiptError> {
    let digest = digest_open_file_handle_bytes_with_buffer(file, buffer)?;
    Ok(FileDigest {
        sha256: bytes_to_hex(&digest.sha256),
        byte_length: digest.byte_length,
    })
}

pub(crate) fn digest_open_file_handle(file: &mut File) -> Result<FileDigest, ProductReceiptError> {
    let digest = digest_open_file_handle_bytes(file)?;
    Ok(FileDigest {
        sha256: bytes_to_hex(&digest.sha256),
        byte_length: digest.byte_length,
    })
}

pub(crate) fn digest_open_file_handle_bytes(
    file: &mut File,
) -> Result<RawFileDigest, ProductReceiptError> {
    let mut buffer = FileDigestBuffer::new();
    digest_open_file_handle_bytes_with_buffer(file, &mut buffer)
}

/// 从调用者已授权且保持打开的句柄取内容身份；游标须在文件起点，调用者还须阻止读取期间写入。
/// 本层只检测长度变化，路径与不可变性准入由捕获或物化层负责。
pub(crate) fn digest_open_file_handle_bytes_with_buffer(
    file: &mut File,
    buffer: &mut FileDigestBuffer,
) -> Result<RawFileDigest, ProductReceiptError> {
    let initial_metadata = file.metadata().map_err(|error| {
        ProductReceiptError::new(format!("could not inspect receipt input file: {error}"))
    })?;
    if !initial_metadata.is_file() {
        return Err(ProductReceiptError::new(
            "product receipt input must be captured from a regular file",
        ));
    }

    let byte_length = initial_metadata.len();
    let mut observed_bytes = 0_u64;
    let mut hasher = Sha256::new();
    loop {
        let count = file.read(&mut buffer.bytes).map_err(|error| {
            ProductReceiptError::new(format!("could not read receipt input file: {error}"))
        })?;
        if count == 0 {
            break;
        }
        observed_bytes = observed_bytes
            .checked_add(count as u64)
            .ok_or_else(|| ProductReceiptError::new("product receipt input length overflowed"))?;
        hasher.update(&buffer.bytes[..count]);
    }
    let final_metadata = file.metadata().map_err(|error| {
        ProductReceiptError::new(format!("could not re-inspect receipt input file: {error}"))
    })?;
    if observed_bytes != byte_length || final_metadata.len() != byte_length {
        return Err(ProductReceiptError::new(
            "product receipt input changed while it was being captured",
        ));
    }

    Ok(RawFileDigest {
        sha256: hasher.finalize().into(),
        byte_length,
    })
}

#[cfg(test)]
#[path = "file_digest/tests/performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "file_digest/tests/cases.rs"]
mod tests;

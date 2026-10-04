//! 收据 CLI 的有界输入读取。
//! 构建、issue、verify 共用同一个字节缓冲，按输入类别限制大小；解析层只有在读完并满足上限后消费切片。

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

const PRODUCT_RECEIPT_READ_CAPACITY: usize = 64 * 1024;

/// CLI 在解析 JSON 或密钥前施加类别大小上限；同一缓冲在连续输入间复用。
pub(super) fn read_bounded<'a>(
    path: &Path,
    limit: usize,
    label: &str,
    contents: &'a mut Vec<u8>,
) -> Result<&'a [u8], io::Error> {
    let file = File::open(path)?;
    read_bounded_from(file, limit, label, contents)
}

fn read_bounded_from<'a>(
    reader: impl Read,
    limit: usize,
    label: &str,
    contents: &'a mut Vec<u8>,
) -> Result<&'a [u8], io::Error> {
    contents.clear();
    contents.reserve(limit.min(PRODUCT_RECEIPT_READ_CAPACITY));
    reader.take(limit as u64 + 1).read_to_end(contents)?;
    if contents.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{label} exceeds the {limit}-byte input limit"),
        ));
    }
    Ok(contents.as_slice())
}

#[cfg(test)]
#[path = "input/tests/performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "tests/input.rs"]
mod tests;

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

const PRODUCT_RECEIPT_READ_CAPACITY: usize = 64 * 1024;

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
mod performance_tests;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::read_bounded;

    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn bounded_read_accepts_the_exact_limit() {
        let path = fixture_path();
        fs::write(&path, b"12345678").unwrap();
        let mut contents = Vec::new();

        let result = read_bounded(&path, 8, "fixture", &mut contents);

        let _ = fs::remove_file(path);
        assert_eq!(result.unwrap(), b"12345678");
    }

    #[test]
    fn bounded_read_rejects_limit_plus_one() {
        let path = fixture_path();
        fs::write(&path, b"123456789").unwrap();
        let mut contents = Vec::new();

        let result = read_bounded(&path, 8, "fixture", &mut contents);

        let _ = fs::remove_file(path);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("8-byte input limit"));
    }

    #[test]
    fn bounded_reads_reuse_existing_capacity() {
        let first_path = fixture_path();
        let second_path = fixture_path();
        fs::write(&first_path, vec![0xA5; 8_192]).unwrap();
        fs::write(&second_path, b"next").unwrap();
        let mut contents = Vec::new();

        read_bounded(&first_path, 16_384, "first", &mut contents).unwrap();
        let first_pointer = contents.as_ptr();
        read_bounded(&second_path, 16_384, "second", &mut contents).unwrap();

        let _ = fs::remove_file(first_path);
        let _ = fs::remove_file(second_path);
        assert_eq!(contents, b"next");
        assert_eq!(contents.as_ptr(), first_pointer);
    }

    fn fixture_path() -> std::path::PathBuf {
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "cargo-zircon-bounded-input-{}-{sequence}.json",
            std::process::id()
        ))
    }
}

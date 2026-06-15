use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::HubError;

const INSTALL_RECEIPT_FILE: &str = "install_receipt.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DeviceInstallReceipt {
    pub format_version: u32,
    pub install_dir: String,
    pub files: Vec<DeviceInstallFileReceipt>,
    pub total_bytes: u64,
    pub content_download_manifest: HubContentDownloadManifest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DeviceInstallFileReceipt {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HubContentDownloadManifest {
    pub download: u64,
    pub resource_id: String,
    pub chunks: Vec<HubContentDownloadChunk>,
    pub mirror_urls: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HubContentDownloadChunk {
    pub id: String,
    pub url: String,
    pub byte_offset: u64,
    pub byte_len: u64,
    pub sha256: String,
    pub resume_from_byte: Option<u64>,
    pub allow_range_resume: bool,
}

pub(super) fn write_install_receipt(
    install_dir: &Path,
) -> Result<(PathBuf, DeviceInstallReceipt), HubError> {
    let receipt = build_install_receipt(install_dir)?;
    let receipt_path = install_dir.join(INSTALL_RECEIPT_FILE);
    fs::write(&receipt_path, serde_json::to_string_pretty(&receipt)?)?;
    Ok((receipt_path, receipt))
}

fn build_install_receipt(install_dir: &Path) -> Result<DeviceInstallReceipt, HubError> {
    let mut files = Vec::new();
    collect_install_files(install_dir, install_dir, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));

    let total_bytes = files.iter().map(|file| file.bytes).sum();
    let content_download_manifest =
        content_download_manifest_for_install(install_dir, &files, total_bytes);

    Ok(DeviceInstallReceipt {
        format_version: 1,
        install_dir: install_dir.to_string_lossy().into_owned(),
        files,
        total_bytes,
        content_download_manifest,
    })
}

fn collect_install_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<DeviceInstallFileReceipt>,
) -> Result<(), HubError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_install_files(root, &path, files)?;
        } else if file_type.is_file() {
            let bytes = fs::read(&path)?;
            let relative_path = install_relative_path(root, &path);
            files.push(DeviceInstallFileReceipt {
                path: relative_path,
                bytes: bytes.len() as u64,
                sha256: sha256_hex(&bytes),
            });
        }
    }
    Ok(())
}

fn content_download_manifest_for_install(
    install_dir: &Path,
    files: &[DeviceInstallFileReceipt],
    total_bytes: u64,
) -> HubContentDownloadManifest {
    let mut offset = 0;
    let chunks = files
        .iter()
        .map(|file| {
            let chunk = HubContentDownloadChunk {
                id: file.path.clone(),
                url: file_url(&install_dir.join(&file.path)),
                byte_offset: offset,
                byte_len: file.bytes,
                sha256: file.sha256.clone(),
                resume_from_byte: None,
                allow_range_resume: true,
            };
            offset += file.bytes;
            chunk
        })
        .collect();

    HubContentDownloadManifest {
        download: stable_download_id(install_dir),
        resource_id: install_dir.to_string_lossy().into_owned(),
        chunks,
        mirror_urls: Vec::new(),
    }
}

fn install_relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn file_url(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy().replace('\\', "/"))
}

fn stable_download_id(path: &Path) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in path.to_string_lossy().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = sha256(bytes);
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

const SHA256_INITIAL_STATE: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut state = SHA256_INITIAL_STATE;
    let mut message = bytes.to_vec();
    let bit_len = (message.len() as u64) * 8;
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in message.chunks(64) {
        let mut schedule = [0u32; 64];
        for (index, word) in schedule.iter_mut().take(16).enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for index in 16..64 {
            let s0 = schedule[index - 15].rotate_right(7)
                ^ schedule[index - 15].rotate_right(18)
                ^ (schedule[index - 15] >> 3);
            let s1 = schedule[index - 2].rotate_right(17)
                ^ schedule[index - 2].rotate_right(19)
                ^ (schedule[index - 2] >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(s1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_ROUND_CONSTANTS[index])
                .wrapping_add(schedule[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut digest = [0u8; 32];
    for (index, word) in state.iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_matches_known_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}

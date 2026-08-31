use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use super::manifest::validate_zrpack_asset_path;
use super::{
    zrpack_content_hash, ZrChunkEntry, ZrPackAssetEntry, ZrPackDocumentManifest, ZrPackError,
    ZrPackManifest, ZRPACK_FORMAT_VERSION, ZRPACK_MAGIC,
};

const ZRPACK_HEADER_SIZE: usize = 24;
const FILE_READ_BUFFER_SIZE: usize = 64 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub struct ZrPackInputAsset {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZrPackWriteReport {
    pub manifest: ZrPackDocumentManifest,
    pub bytes: Vec<u8>,
    pub deduplicated_assets: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ZrPackWriter;

#[derive(Debug)]
pub(crate) enum ZrPackFileWriteError {
    Pack(ZrPackError),
    ReadSource { path: PathBuf, source: io::Error },
    SourceChanged { path: PathBuf },
}

impl fmt::Display for ZrPackFileWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pack(error) => error.fmt(formatter),
            Self::ReadSource { path, source } => {
                write!(
                    formatter,
                    "failed to read asset source {}: {source}",
                    path.display()
                )
            }
            Self::SourceChanged { path } => write!(
                formatter,
                "asset source changed while streaming {}",
                path.display()
            ),
        }
    }
}

impl From<ZrPackError> for ZrPackFileWriteError {
    fn from(error: ZrPackError) -> Self {
        Self::Pack(error)
    }
}

impl ZrPackInputAsset {
    pub fn new(path: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            bytes: bytes.into(),
        }
    }
}

impl ZrPackWriter {
    pub fn write<I, A>(assets: I) -> Result<ZrPackWriteReport, ZrPackError>
    where
        I: IntoIterator<Item = A>,
        A: Borrow<ZrPackInputAsset>,
    {
        let mut assets = assets.into_iter().collect::<Vec<_>>();
        validate_asset_paths(&assets)?;
        sort_assets_by_path(&mut assets);
        reject_duplicate_paths(&assets)?;

        let mut assembler = ZrPackAssembler::new(assets.len());

        for asset in &assets {
            let asset = input_asset(asset);
            assembler.push_bytes(&asset.path, &asset.bytes)?;
        }

        assembler.finish()
    }

    pub(crate) fn write_files<I, P, S>(assets: I) -> Result<ZrPackWriteReport, ZrPackFileWriteError>
    where
        I: IntoIterator<Item = (P, S)>,
        P: AsRef<str>,
        S: AsRef<Path>,
    {
        let mut assets = assets
            .into_iter()
            .map(|(path, source)| (path.as_ref().to_string(), source.as_ref().to_path_buf()))
            .collect::<Vec<_>>();
        for (path, _) in &assets {
            validate_zrpack_asset_path(path)?;
        }
        assets.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        if let Some(pair) = assets.windows(2).find(|pair| pair[0].0 == pair[1].0) {
            return Err(ZrPackError::DuplicateAssetPath(pair[1].0.clone()).into());
        }

        let mut assembler = ZrPackAssembler::new(assets.len());
        for (path, source) in assets {
            assembler.push_file(&path, &source)?;
        }
        assembler.finish().map_err(Into::into)
    }
}

struct ZrPackAssembler {
    bytes: Vec<u8>,
    chunk_offsets: BTreeMap<[u8; 32], u64>,
    chunk_entries: Vec<ZrChunkEntry>,
    asset_entries: Vec<ZrPackAssetEntry>,
    deduplicated_assets: Vec<String>,
}

impl ZrPackAssembler {
    fn new(asset_count: usize) -> Self {
        Self {
            bytes: vec![0; ZRPACK_HEADER_SIZE],
            chunk_offsets: BTreeMap::new(),
            chunk_entries: Vec::with_capacity(asset_count),
            asset_entries: Vec::with_capacity(asset_count),
            deduplicated_assets: Vec::with_capacity(asset_count),
        }
    }

    fn push_bytes(&mut self, path: &str, payload: &[u8]) -> Result<(), ZrPackError> {
        let hash = zrpack_content_hash(payload);
        let size = u32::try_from(payload.len()).map_err(|_| ZrPackError::SizeOverflow)?;
        if self.record_deduplicated(path, hash, u64::from(size)) {
            return Ok(());
        }

        let offset = u64::try_from(self.bytes.len()).map_err(|_| ZrPackError::SizeOverflow)?;
        self.bytes.extend_from_slice(payload);
        self.record_unique(path, hash, offset, size);
        Ok(())
    }

    fn push_file(&mut self, path: &str, source: &Path) -> Result<(), ZrPackFileWriteError> {
        let mut file = File::open(source).map_err(|error| ZrPackFileWriteError::ReadSource {
            path: source.to_path_buf(),
            source: error,
        })?;
        let (hash, payload_size) = scan_file(&mut file, source, None, |_| {})?;
        if self.record_deduplicated(path, hash, payload_size) {
            return Ok(());
        }

        file.seek(SeekFrom::Start(0))
            .map_err(|error| ZrPackFileWriteError::ReadSource {
                path: source.to_path_buf(),
                source: error,
            })?;
        let payload_offset = self.bytes.len();
        let (written_hash, written_size) =
            scan_file(&mut file, source, Some(payload_size), |chunk| {
                self.bytes.extend_from_slice(chunk);
            })?;
        if written_hash != hash || written_size != payload_size {
            return Err(ZrPackFileWriteError::SourceChanged {
                path: source.to_path_buf(),
            });
        }

        let offset = u64::try_from(payload_offset).map_err(|_| ZrPackError::SizeOverflow)?;
        let size = u32::try_from(payload_size).map_err(|_| ZrPackError::SizeOverflow)?;
        self.record_unique(path, hash, offset, size);
        Ok(())
    }

    fn record_deduplicated(&mut self, path: &str, hash: [u8; 32], size: u64) -> bool {
        let Some(offset) = self.chunk_offsets.get(&hash).copied() else {
            return false;
        };
        self.deduplicated_assets.push(path.to_string());
        self.asset_entries
            .push(ZrPackAssetEntry::new(path, hash, size));
        debug_assert!(offset <= self.bytes.len() as u64);
        true
    }

    fn record_unique(&mut self, path: &str, hash: [u8; 32], offset: u64, size: u32) {
        self.chunk_offsets.insert(hash, offset);
        self.chunk_entries
            .push(ZrChunkEntry::new(hash, offset, size));
        self.asset_entries
            .push(ZrPackAssetEntry::new(path, hash, u64::from(size)));
    }

    fn finish(mut self) -> Result<ZrPackWriteReport, ZrPackError> {
        self.chunk_entries
            .sort_unstable_by(|left, right| left.hash.cmp(&right.hash));
        let total_size = self
            .chunk_entries
            .iter()
            .map(|chunk| u64::from(chunk.size))
            .sum();
        let manifest = ZrPackDocumentManifest::new(
            ZrPackManifest {
                version: ZRPACK_FORMAT_VERSION,
                chunks: self.chunk_entries,
                total_size,
            },
            self.asset_entries,
        );
        let manifest_bytes = serde_json::to_vec(&manifest)
            .map_err(|error| ZrPackError::ManifestDecode(error.to_string()))?;
        let manifest_offset =
            u64::try_from(self.bytes.len()).map_err(|_| ZrPackError::SizeOverflow)?;
        let manifest_size =
            u64::try_from(manifest_bytes.len()).map_err(|_| ZrPackError::SizeOverflow)?;
        self.bytes.extend_from_slice(&manifest_bytes);
        write_header(
            &mut self.bytes[..ZRPACK_HEADER_SIZE],
            manifest_offset,
            manifest_size,
        );

        Ok(ZrPackWriteReport {
            manifest,
            bytes: self.bytes,
            deduplicated_assets: self.deduplicated_assets,
        })
    }
}

fn scan_file(
    file: &mut File,
    source: &Path,
    expected_size: Option<u64>,
    mut consume: impl FnMut(&[u8]),
) -> Result<([u8; 32], u64), ZrPackFileWriteError> {
    let mut payload_size = 0_u64;
    let mut hasher = blake3::Hasher::new();
    let mut read_buffer = [0_u8; FILE_READ_BUFFER_SIZE];
    loop {
        let read = match file.read(&mut read_buffer) {
            Ok(read) => read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                return Err(ZrPackFileWriteError::ReadSource {
                    path: source.to_path_buf(),
                    source: error,
                });
            }
        };
        if read == 0 {
            break;
        }
        payload_size = payload_size
            .checked_add(read as u64)
            .ok_or(ZrPackError::SizeOverflow)?;
        if payload_size > u64::from(u32::MAX) {
            return Err(ZrPackError::SizeOverflow.into());
        }
        if expected_size.is_some_and(|expected| payload_size > expected) {
            return Err(ZrPackFileWriteError::SourceChanged {
                path: source.to_path_buf(),
            });
        }
        hasher.update(&read_buffer[..read]);
        consume(&read_buffer[..read]);
    }
    if expected_size.is_some_and(|expected| payload_size != expected) {
        return Err(ZrPackFileWriteError::SourceChanged {
            path: source.to_path_buf(),
        });
    }
    Ok((*hasher.finalize().as_bytes(), payload_size))
}

fn input_asset<A>(asset: &A) -> &ZrPackInputAsset
where
    A: Borrow<ZrPackInputAsset>,
{
    asset.borrow()
}

fn sort_assets_by_path<A>(assets: &mut [A])
where
    A: Borrow<ZrPackInputAsset>,
{
    assets.sort_unstable_by(|left, right| input_asset(left).path.cmp(&input_asset(right).path));
}

fn validate_asset_paths<A>(assets: &[A]) -> Result<(), ZrPackError>
where
    A: Borrow<ZrPackInputAsset>,
{
    for asset in assets {
        let asset = input_asset(asset);
        validate_zrpack_asset_path(&asset.path)?;
    }
    Ok(())
}

fn reject_duplicate_paths<A>(assets: &[A]) -> Result<(), ZrPackError>
where
    A: Borrow<ZrPackInputAsset>,
{
    if let Some(pair) = assets
        .windows(2)
        .find(|pair| input_asset(&pair[0]).path == input_asset(&pair[1]).path)
    {
        return Err(ZrPackError::DuplicateAssetPath(
            input_asset(&pair[1]).path.clone(),
        ));
    }
    Ok(())
}

fn write_header(header: &mut [u8], manifest_offset: u64, manifest_size: u64) {
    header[0..4].copy_from_slice(&ZRPACK_MAGIC);
    header[4..8].copy_from_slice(&ZRPACK_FORMAT_VERSION.to_le_bytes());
    header[8..16].copy_from_slice(&manifest_offset.to_le_bytes());
    header[16..24].copy_from_slice(&manifest_size.to_le_bytes());
}

pub(super) fn header_size() -> usize {
    ZRPACK_HEADER_SIZE
}

#[cfg(test)]
#[path = "writer/optimization_tests.rs"]
mod optimization_tests;

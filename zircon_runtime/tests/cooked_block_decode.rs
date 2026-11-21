//! Behavioral port of UE `Core::Compression::CompressedBuffer::Decompress`:
//! a complete block yields the original bytes and a truncated block yields no payload.
#![cfg(all(feature = "cooked-assets", not(target_arch = "wasm32")))]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use zircon_runtime::asset::artifact::{
    RenderArtifactBlockCodec, RenderArtifactBlockDescriptor, RenderArtifactBlockFailureCode,
    RenderArtifactBlockIoDispatchBudget, RenderArtifactBlockLoader,
    RenderArtifactBlockLoaderLimits, RenderArtifactBlockPoll, RenderArtifactBlockRequest,
    RenderArtifactContentId, RenderArtifactIoPriority, RenderArtifactResidencyClass,
    RenderArtifactStore, RenderArtifactStoreLimits, RenderSubresourceId,
};
use zircon_runtime::core::runtime::{EngineTaskGraph, EngineTaskGraphOptions};

#[test]
fn cooked_block_load_decodes_complete_and_rejects_truncated_frames() {
    let root = BlockFixture::new();
    let budget = Duration::from_secs(5);
    let graph = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
    let loader = RenderArtifactBlockLoader::new(
        RenderArtifactStore::new(root.0.clone()),
        RenderArtifactBlockLoaderLimits::new(
            4,
            4,
            4,
            16 * 1024,
            8 * 1024,
            RenderArtifactStoreLimits::new(1024, 8 * 1024),
        ),
        &graph,
    )
    .unwrap();
    let decoded: Vec<u8> = (0..4096).map(|index| (index % 251) as u8).collect();
    let frame = zstd::stream::encode_all(decoded.as_slice(), 1).unwrap();

    for (encoded, truncated) in [(frame.as_slice(), false), (&frame[..frame.len() - 1], true)] {
        let descriptor = seed_cooked_block(&root.0, encoded, decoded.len());
        let ticket = loader
            .request(RenderArtifactBlockRequest::new(
                descriptor,
                RenderArtifactIoPriority::NORMAL,
            ))
            .unwrap();
        loader
            .dispatch_io(RenderArtifactBlockIoDispatchBudget::new(1, 8 * 1024))
            .unwrap();
        let deadline = Instant::now() + budget;
        let terminal = loop {
            let poll = ticket.poll();
            if !matches!(poll, RenderArtifactBlockPoll::Pending(_)) {
                break poll;
            }
            assert!(Instant::now() < deadline, "cooked block did not finish");
            std::thread::sleep(Duration::from_millis(1));
        };
        match (truncated, terminal) {
            (false, RenderArtifactBlockPoll::Ready(block)) => {
                assert_eq!(block.bytes().as_ref(), decoded.as_slice());
            }
            (true, RenderArtifactBlockPoll::Failed(failure)) => {
                assert_eq!(failure.code(), RenderArtifactBlockFailureCode::DecodeFailed);
            }
            (truncated, other) => {
                panic!("unexpected cooked block result (truncated={truncated}): {other:?}")
            }
        }
    }

    let _ = loader.close();
    assert!(!graph.shutdown(budget).unwrap().has_in_flight_work());
}

fn seed_cooked_block(
    root: &Path,
    encoded: &[u8],
    decoded_len: usize,
) -> RenderArtifactBlockDescriptor {
    // Supply the staged layout: the portable loader does not own the native-host writer.
    // Both variants have a matching on-disk size and hash, so rejection must come from decoding.
    let hash = blake3::hash(encoded);
    let hex = hash.to_hex();
    let directory = root.join("render-artifacts").join("blocks").join(&hex[..2]);
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join(format!("{hex}.zr-render-block")), encoded).unwrap();
    RenderArtifactBlockDescriptor::new(
        RenderSubresourceId::TextureMipLayer { mip: 0, layer: 0 },
        RenderArtifactContentId::from_bytes(*hash.as_bytes()),
        RenderArtifactBlockCodec::Zstd,
        encoded.len() as u64,
        decoded_len as u64,
        256,
        Arc::from("rgba8unorm"),
        RenderArtifactResidencyClass::Bootstrap,
        Vec::new(),
    )
}

struct BlockFixture(PathBuf);

impl BlockFixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("ze-block-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for BlockFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

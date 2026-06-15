pub mod framework {
    pub mod net {
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        pub struct ZrPackManifest {
            pub version: u32,
            pub chunks: Vec<ZrChunkEntry>,
            pub total_size: u64,
        }

        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        pub struct ZrChunkEntry {
            pub hash: [u8; 32],
            pub offset: u64,
            pub size: u32,
        }

        impl ZrChunkEntry {
            pub fn new(hash: [u8; 32], offset: u64, size: u32) -> Self {
                Self { hash, offset, size }
            }
        }
    }
}

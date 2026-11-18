use std::fs;
use std::path::Path;

use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetUri, ImportedAsset};
use zircon_runtime::core::framework::audio::AudioChannelLayout;

use super::setup;

const SAMPLE_RATE: u32 = 8_000;

pub(super) struct Fixture {
    pub(super) path: &'static str,
    pub(super) bits: [u32; 3],
    pub(super) sha256: &'static str,
}

pub(super) const FIXTURES: [Fixture; 2] = [
    Fixture {
        path: "audio/nan.wav",
        bits: [0, 0x7fc0_0000, 0.25_f32.to_bits()],
        sha256: "43126fd051965c37a6016927007abf7f8b9d7cc6cfad11a03f657c5d8988a040",
    },
    Fixture {
        path: "audio/finite.wav",
        bits: [0, 0.125_f32.to_bits(), 0.25_f32.to_bits()],
        sha256: "19befa7552971eab18bec758118997ac3e43a25a28963cca3249ae4ab3282e63",
    },
];

impl Fixture {
    pub(super) fn uri(&self) -> AssetUri {
        setup(
            AssetUri::parse(&format!("res://{}", self.path)),
            "fixture URI",
        )
    }

    pub(super) fn wav(&self) -> Vec<u8> {
        // RIFF size 62, IEEE float tag 3 / mono / 8000 Hz / 32-bit / cbSize 0.
        // fact = 3 frames, data = 12 bytes. The two inputs differ only in sample slot 1.
        let header: &[u8; 58] = b"RIFF\x3e\0\0\0WAVE\
            fmt \x12\0\0\0\x03\0\x01\0\x40\x1f\0\0\0\x7d\0\0\x04\0\x20\0\0\0\
            fact\x04\0\0\0\x03\0\0\0data\x0c\0\0\0";
        let mut bytes = header.to_vec();
        for bits in self.bits {
            bytes.extend_from_slice(&bits.to_le_bytes());
        }
        bytes
    }
}

pub(super) fn create_project(root: &Path) {
    eprintln!("wav156 setup new project {root:?}");
    setup(fs::create_dir(root), "create independent project root");
    let paths = setup(ProjectPaths::from_root(root), "resolve new project paths");
    assert_eq!(paths.root(), root, "setup project root alias");
    let project = ProjectManifest::new("wav156", FIXTURES[0].uri(), 1);
    setup(
        paths.ensure_layout(&project.asset_roots),
        "create project layout",
    );
    setup(
        project.save(paths.manifest_path()),
        "write project manifest",
    );
    let assets = setup(
        project.primary_asset_root_path(&paths),
        "primary asset root",
    );
    setup(
        fs::create_dir(assets.join("audio")),
        "create audio directory",
    );
    for fixture in &FIXTURES {
        setup(
            fs::write(assets.join(fixture.path), fixture.wav()),
            "write 70-byte WAV",
        );
    }
    assert_eq!(
        setup(
            fs::read_dir(paths.asset_artifact_root()),
            "inspect new store"
        )
        .count(),
        0
    );
}

pub(super) fn pcm_bits(asset: ImportedAsset, uri: &AssetUri) -> Vec<u32> {
    let ImportedAsset::Sound(sound) = asset else {
        panic!("expected decoded Sound at {uri}, got {asset:?}");
    };
    assert_eq!(&sound.uri, uri);
    assert_eq!(sound.sample_rate_hz, SAMPLE_RATE);
    assert_eq!(sound.channel_count, 1);
    assert_eq!(sound.channel_layout, AudioChannelLayout::mono());
    assert_eq!(sound.frame_count(), 3);
    assert_eq!(sound.samples.len(), 3);
    sound.samples.into_iter().map(f32::to_bits).collect()
}

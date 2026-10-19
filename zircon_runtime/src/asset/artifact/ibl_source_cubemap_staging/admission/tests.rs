use std::cell::{Cell, RefCell};
use std::path::Path;

thread_local! {
    static READS: Cell<usize> = const { Cell::new(0) };
    static AFTER_READ: RefCell<Option<Box<dyn FnMut(&Path)>>> = const { RefCell::new(None) };
}

pub(super) fn observe_payload_read(path: &Path) {
    READS.with(|reads| reads.set(reads.get() + 1));
    let hook = AFTER_READ.with(|hook| hook.borrow_mut().take());
    if let Some(mut hook) = hook {
        hook(path);
    }
}

#[cfg(any(windows, unix))]
mod supported {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::super::*;
    use super::{AFTER_READ, READS};
    use crate::asset::AssetUri;
    use crate::core::framework::render::{
        IblBakeArtifactContents, IblBakeKey, SourceCubemapMipChain, SourceCubemapPrefilterQuality,
    };

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        store: IblSourceCubemapStagingStore,
        request: IblBakeArtifactRequest,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "zircon-ibl-admission-{label}-{}-{}",
                std::process::id(),
                NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&root).expect("registered fixture root must be writable");
            let store = IblSourceCubemapStagingStore::new(&root);
            let request = IblBakeArtifactRequest::new(IblBakeKey::source_cubemap(7, [11; 4]), 4, 3)
                .with_pmrem_layout(2, 2)
                .with_required_contents(IblBakeArtifactContents::PMREM_SH9);
            let source = SourceCubemapMipChain::from_equirect_with_pmrem_layout(
                4,
                2,
                2,
                SourceCubemapPrefilterQuality::Fast,
                |_, _| [0.25, 0.5, 0.75, 1.0],
            );
            store
                .write_source_cubemap_staged_bundle(
                    &request,
                    AssetUri::parse("res://environment/admission.hdr").unwrap(),
                    &source,
                    None,
                )
                .expect("real bundle publication and disk admission must succeed");
            Self {
                root,
                store,
                request,
            }
        }

        fn admission(&self) -> BundleAdmission {
            self.store
                .admit_bundle_for_staging(&self.request, IblSourceImageIdentity::default())
                .expect("stable fixture admission must be observable")
        }

        fn hot(&self) -> bool {
            IblSourceCubemapStagingStore::new(&self.root)
                .current_bundle_manifest_matches(&self.request, IblSourceImageIdentity::default())
                .expect("metadata probe must succeed")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = invalidate_root(&self.store);
            let result = fs::remove_dir_all(&self.root);
            if !std::thread::panicking() {
                result.expect("owned admission fixture must be removable");
            }
        }
    }

    fn corrupt(path: &Path) {
        let mut bytes = fs::read(path).unwrap();
        let index = bytes.len() - 1;
        bytes[index] ^= 1;
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn publication_seeds_actual_disk_and_new_store_hot_probes_do_not_read_payloads() {
        READS.with(|reads| reads.set(0));
        let fixture = Fixture::new("seed-hot");
        assert!(
            READS.with(|reads| reads.get()) >= 2,
            "publication must verify both actual files"
        );
        READS.with(|reads| reads.set(0));
        for _ in 0..12 {
            assert!(
                fixture.hot(),
                "new Store values must share verified process credentials"
            );
        }
        assert_eq!(
            READS.with(|reads| reads.get()),
            0,
            "hot proof must never read or hash payloads"
        );
    }

    #[test]
    fn equal_length_source_corruption_is_missing_and_derived_corruption_keeps_only_verified_source()
    {
        for source_bad in [true, false] {
            let fixture = Fixture::new(if source_bad {
                "bad-source"
            } else {
                "bad-derived"
            });
            let path = if source_bad {
                fixture.store.source_cubemap_path(&fixture.request)
            } else {
                fixture
                    .store
                    .asset_derived_store()
                    .asset_derived_path(&fixture.request)
            };
            let length = fs::metadata(&path).unwrap().len();
            corrupt(&path);
            assert_eq!(fs::metadata(&path).unwrap().len(), length);
            assert!(!fixture.hot());
            match fixture.admission() {
                BundleAdmission::Missing if source_bad => {}
                BundleAdmission::VerifiedSourceOnly(_, _) if !source_bad => {}
                _ => panic!("corruption must not become Current or an unverified source reuse"),
            }
            assert!(!fixture.hot());
        }
    }

    #[test]
    fn same_object_overwrite_with_restored_mtime_invalidates_change_identity() {
        let fixture = Fixture::new("restore-mtime");
        let path = fixture.store.source_cubemap_path(&fixture.request);
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        let mut file = fs::OpenOptions::new().write(true).open(&path).unwrap();
        file.write_all(b"!").unwrap();
        file.sync_all().unwrap();
        file.set_modified(modified).unwrap();
        drop(file);
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
        assert!(
            !fixture.hot(),
            "file identity plus change time must reject a restored mtime"
        );
        assert!(matches!(fixture.admission(), BundleAdmission::Missing));
    }

    #[test]
    fn replacement_and_manifest_changes_cannot_reuse_old_credentials() {
        let fixture = Fixture::new("replacement");
        let path = fixture.store.source_cubemap_path(&fixture.request);
        let replacement = path.with_extension("replacement");
        fs::write(&replacement, fs::read(&path).unwrap()).unwrap();
        fs::remove_file(&path).unwrap();
        fs::rename(&replacement, &path).unwrap();
        assert!(
            !fixture.hot(),
            "identical bytes at a new file object require admission"
        );
        assert!(fixture.admission().is_current());
        assert!(fixture.hot());
        let manifest_path = fixture.store.bundle_manifest_path(&fixture.request);
        let manifest = fs::read(&manifest_path).unwrap();
        corrupt(&manifest_path);
        assert!(!fixture.hot());
        assert!(matches!(fixture.admission(), BundleAdmission::Missing));
        fs::write(&manifest_path, manifest).unwrap();
        assert!(fixture.admission().is_current());
    }

    #[test]
    fn pending_publication_invalidates_credentials_before_recovery_or_probe() {
        let fixture = Fixture::new("pending");
        let pending = fixture
            .store
            .bundle_journal_directory()
            .join("unrecognized-test-entry");
        fs::write(&pending, b"explicit synthetic pending evidence").unwrap();
        assert!(!matches!(
            fixture.store.current_bundle_manifest_matches(
                &fixture.request,
                IblSourceImageIdentity::default()
            ),
            Ok(true)
        ));
        fs::remove_file(pending).unwrap();
        assert!(
            !fixture.hot(),
            "settled journal must not revive an invalidated credential"
        );
        assert!(fixture.admission().is_current());
    }

    #[test]
    fn same_file_change_during_payload_read_cannot_publish_a_credential() {
        let fixture = Fixture::new("read-race");
        invalidate_root(&fixture.store).unwrap();
        AFTER_READ.with(|hook| *hook.borrow_mut() = Some(Box::new(|path| corrupt(path))));
        let observed = fixture
            .store
            .admit_bundle_for_staging(&fixture.request, IblSourceImageIdentity::default());
        assert!(!matches!(observed, Ok(BundleAdmission::Current)));
        assert!(!fixture.hot());
    }

    #[test]
    fn owned_source_proof_survives_eviction_but_rejects_changed_source_or_manifest() {
        for mutate in [false, true] {
            let fixture = Fixture::new(if mutate {
                "proof-change"
            } else {
                "proof-eviction"
            });
            corrupt(
                &fixture
                    .store
                    .asset_derived_store()
                    .asset_derived_path(&fixture.request),
            );
            let BundleAdmission::VerifiedSourceOnly(_, proof) = fixture.admission() else {
                panic!("source-only admission must own its original proof");
            };
            invalidate_root(&fixture.store).unwrap();
            if mutate {
                corrupt(&fixture.store.source_cubemap_path(&fixture.request));
            }
            let prepared = fixture.store.prepare_bundle_manifest_for_existing_source(
                &fixture.request,
                IblSourceImageIdentity::default(),
                b"synthetic derived preparation",
                proof,
            );
            if mutate {
                assert!(matches!(
                    prepared,
                    Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication)
                ));
            } else {
                assert!(
                    prepared.is_ok(),
                    "owned proof must not depend on a cache entry"
                );
            }
        }
        let fixture = Fixture::new("proof-manifest");
        corrupt(
            &fixture
                .store
                .asset_derived_store()
                .asset_derived_path(&fixture.request),
        );
        let BundleAdmission::VerifiedSourceOnly(_, proof) = fixture.admission() else {
            panic!("proof required");
        };
        corrupt(&fixture.store.bundle_manifest_path(&fixture.request));
        assert!(matches!(
            fixture.store.prepare_bundle_manifest_for_existing_source(
                &fixture.request,
                IblSourceImageIdentity::default(),
                b"synthetic derived preparation",
                proof
            ),
            Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication)
        ));
    }

    #[test]
    fn process_credentials_are_bounded_and_evicted_bundles_require_new_admission() {
        let fixtures = (0..65).map(|_| Fixture::new("bounded")).collect::<Vec<_>>();
        assert!(cache().unwrap().len() <= MAX_CREDENTIALS);
        assert!(
            !fixtures[0].hot(),
            "65 real published roots must evict the oldest credential"
        );
        assert!(fixtures[0].admission().is_current());
        assert!(fixtures[0].hot());
        assert!(cache().unwrap().len() <= MAX_CREDENTIALS);
    }
}

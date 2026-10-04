use crate::service::{
    cloud::{manifest::FileEntry, BlobStore, CloudConfig, CommitRequest, Manifest},
    identity::Principal,
    organization,
    storage::Database,
    test_support::make_private_directory,
};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, sync::Arc};
use zircon_runtime_interface::project::{
    ProjectGuid, ProjectManifestDigest, ProjectPackageLock, ProjectPackageLockAuthority,
    ProjectPackageLockPlatform, ProjectPackageLockProject, ProjectPackageLockRuntimeMode,
    ProjectPackageLockState, ProjectPackageLockTarget, PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
};

pub struct Files {
    pub path: PathBuf,
    pub config: CloudConfig,
}

impl Files {
    pub fn new() -> Self {
        let path = std::env::temp_dir().join(format!("zircon-cloud-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&path).unwrap();
        make_private_directory(&path);
        let config = CloudConfig {
            root: path.join("cas"),
            key_file: path.join("cloud.key"),
        };
        fs::write(&config.key_file, [7u8; 32]).unwrap();
        Self { path, config }
    }

    pub async fn database(&self) -> (Database, Arc<BlobStore>) {
        let database = Database::open(self.path.join("service.db")).unwrap();
        let config = self.config.clone();
        let store = database
            .execute(move |connection| {
                let store = BlobStore::load(&config)?;
                store.recover(connection)?;
                Ok(Arc::new(store))
            })
            .await
            .unwrap();
        (database, store)
    }
}

impl Drop for Files {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn principal(subject: &str) -> Principal {
    Principal {
        issuer: "https://issuer.test".into(),
        subject: subject.into(),
    }
}

pub fn seed(connection: &mut Connection, owner: &Principal) -> (String, String) {
    let organization = organization::create(
        connection,
        owner,
        &uuid::Uuid::new_v4().to_string(),
        "same-name",
    )
    .unwrap()
    .id;
    let project = "same-project".to_string();
    connection
        .execute(
            "INSERT INTO projects VALUES (?1,?2,?3)",
            params![organization, project, "same-name"],
        )
        .unwrap();
    (organization, project)
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn manifest(bytes: &[u8]) -> Manifest {
    let lock = ProjectPackageLock {
        schema_version: PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
        project: ProjectPackageLockProject {
            project_guid: ProjectGuid::new(),
            manifest_digest: ProjectManifestDigest::from_bytes(b"service fixture"),
        },
        target: ProjectPackageLockTarget {
            runtime_mode: ProjectPackageLockRuntimeMode::EditorHost,
            platform: ProjectPackageLockPlatform::Windows,
        },
        authority: ProjectPackageLockAuthority {
            index_sha256: "a".repeat(64),
            policy_sha256: "b".repeat(64),
            build_set_id: "c".repeat(64),
            capability_digest: "d".repeat(64),
            provider_revision_digest: "e".repeat(64),
        },
        entries: Vec::new(),
    };
    let package_lock_digest = lock.digest().unwrap();
    Manifest {
        schema_version: 1,
        engine: "zircon-test".into(),
        package_lock_digest,
        package_lock: Some(ProjectPackageLockState::present(lock).unwrap()),
        ignore_policy: "zircon-project-v1".into(),
        source_revision: Some("main".into()),
        files: vec![FileEntry {
            path: "Assets/scene.zr".into(),
            digest: digest(bytes),
            bytes: bytes.len() as u64,
        }],
    }
}

pub fn request(base: &str, bytes: &[u8]) -> CommitRequest {
    CommitRequest {
        operation_id: uuid::Uuid::new_v4().to_string(),
        base_revision: base.into(),
        manifest: manifest(bytes),
    }
}

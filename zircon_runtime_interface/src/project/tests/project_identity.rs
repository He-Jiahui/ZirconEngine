use std::path::PathBuf;
use std::str::FromStr;

use super::super::{
    CanonicalDescriptorIdentity, ProjectGuid, ProjectIdentity, ProjectManifestDigest,
};

#[test]
fn project_identity_preserves_canonical_descriptor_guid_and_manifest_digest() {
    let canonical_path = std::env::temp_dir().join("zircon-project-identity-contract");
    let descriptor = CanonicalDescriptorIdentity::new(canonical_path.clone()).unwrap();
    let project_guid = ProjectGuid::from_str("62449228-b3e3-482e-b6d9-7dc59cf8c980").unwrap();
    let manifest_digest = ProjectManifestDigest::from_bytes(b"project identity fixture");

    let identity = ProjectIdentity::new(descriptor.clone(), project_guid, manifest_digest);

    assert_eq!(identity.canonical_descriptor(), &descriptor);
    assert_eq!(identity.canonical_descriptor().path(), canonical_path);
    assert_eq!(identity.project_guid(), project_guid);
    assert_eq!(identity.manifest_digest(), manifest_digest);
    assert_eq!(
        serde_json::from_str::<ProjectIdentity>(&serde_json::to_string(&identity).unwrap())
            .unwrap(),
        identity
    );
}

#[test]
fn canonical_descriptor_identity_rejects_nonphysical_path_shapes() {
    assert!(CanonicalDescriptorIdentity::new(PathBuf::from("relative-project")).is_err());
    assert!(CanonicalDescriptorIdentity::new(
        std::env::temp_dir()
            .join("zircon-project-identity-contract")
            .join(".."),
    )
    .is_err());
    assert!(serde_json::from_str::<CanonicalDescriptorIdentity>("\"relative-project\"").is_err());
}

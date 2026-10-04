//! 验证 descriptor 物理路径、项目 GUID 与清单摘要哈希组成的可持久身份。

use std::path::PathBuf;
use std::str::FromStr;

use super::super::{
    CanonicalDescriptorIdentity, ProjectGuid, ProjectIdentity, ProjectManifestDigest,
};

// 序列化往返必须保留三项身份；后续 preflight 以它们判定是否仍是同一项目。
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

// 相对路径和 dot segment 会引入词法别名，构造和反序列化都拒绝这些未解析形态。
#[test]
fn canonical_descriptor_identity_rejects_nonphysical_path_shapes() {
    assert!(CanonicalDescriptorIdentity::new(PathBuf::from("relative-project")).is_err());
    let roots = [
        std::env::temp_dir().join("zircon-project-identity-contract"),
        #[cfg(windows)]
        PathBuf::from(r"C:\zircon-project-identity-contract"),
    ];
    for root in roots {
        for segment in [".", ".."] {
            // PathBuf::push normalizes dot segments in Windows verbatim paths.
            let mut path = root.clone().into_os_string();
            path.push(std::path::MAIN_SEPARATOR_STR);
            path.push(segment);
            let path = PathBuf::from(path);
            let encoded = serde_json::to_string(&path).unwrap();
            assert!(serde_json::from_str::<CanonicalDescriptorIdentity>(&encoded).is_err());
            assert!(CanonicalDescriptorIdentity::new(path).is_err());
        }
    }
    assert!(serde_json::from_str::<CanonicalDescriptorIdentity>("\"relative-project\"").is_err());
}

#[test]
fn canonical_descriptor_identity_preserves_literal_filename_dots() {
    let path = std::env::temp_dir().join(".zircon").join("project..toml");
    assert_eq!(
        CanonicalDescriptorIdentity::new(path.clone())
            .unwrap()
            .path(),
        path
    );
}

#[cfg(windows)]
#[test]
fn canonical_descriptor_identity_preserves_non_unicode_windows_paths() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let path = PathBuf::from(OsString::from_wide(&[
        b'C' as u16,
        b':' as u16,
        b'\\' as u16,
        0xD800,
    ]));
    assert_eq!(
        CanonicalDescriptorIdentity::new(path.clone())
            .unwrap()
            .path(),
        path
    );
}

use std::path::PathBuf;

use super::{profile_output_root, ProfileOutputRootError};

#[test]
fn profile_output_root_requires_an_absolute_non_c_drive_path() {
    assert_eq!(
        profile_output_root(r"E:\zircon-profiles"),
        Ok(PathBuf::from(r"E:\zircon-profiles"))
    );
    assert_eq!(
        profile_output_root(r"e:/zircon-profiles"),
        Ok(PathBuf::from(r"e:/zircon-profiles"))
    );
    assert_eq!(
        profile_output_root(r"\\artifact-host\zircon-profiles"),
        Ok(PathBuf::from(r"\\artifact-host\zircon-profiles"))
    );
    assert_eq!(
        profile_output_root(r"C:\zircon-profiles"),
        Err(ProfileOutputRootError)
    );
    assert_eq!(
        profile_output_root(r"\\?\C:\zircon-profiles"),
        Err(ProfileOutputRootError)
    );
    assert_eq!(
        profile_output_root(r"\\.\C:\zircon-profiles"),
        Err(ProfileOutputRootError)
    );
    assert_eq!(
        profile_output_root(r"c:/zircon-profiles"),
        Err(ProfileOutputRootError)
    );
    assert_eq!(
        profile_output_root(r"zircon-profiles"),
        Err(ProfileOutputRootError)
    );
    assert_eq!(
        profile_output_root(r"E:zircon-profiles"),
        Err(ProfileOutputRootError)
    );
}

use super::*;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("native-owner-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn recovers_abandoned_generation_but_keeps_live_and_foreign_directories() {
    let fixture = Fixture::new();
    let live = StagingDirectory::create(&fixture.0).unwrap();
    let live_path = live.path().to_path_buf();
    let mut abandoned = StagingDirectory::create(&fixture.0).unwrap();
    let abandoned_path = abandoned.path().to_path_buf();
    fs::write(abandoned_path.join("plugin.dll"), b"fixture").unwrap();
    // Model process death: kernel releases the lease without running cleanup.
    drop(abandoned.owner.take());
    let foreign = fixture.0.join(PARENT).join(uuid::Uuid::new_v4().to_string());
    fs::create_dir(&foreign).unwrap();
    fs::write(foreign.join("plugin.dll"), b"foreign").unwrap();
    let malformed = fixture.0.join(PARENT).join(uuid::Uuid::new_v4().to_string());
    fs::create_dir(&malformed).unwrap();
    fs::write(malformed.join(OWNER), b"not our owner marker").unwrap();
    let next = StagingDirectory::create(&fixture.0).unwrap();
    assert!(!abandoned_path.exists());
    assert!(live_path.is_dir());
    assert!(foreign.join("plugin.dll").is_file());
    assert!(malformed.is_dir());
    drop(abandoned);
    drop(live);
    assert!(!live_path.exists());
    drop(next);
}

#[cfg(unix)]
#[test]
fn sweep_retains_symlinked_generations_and_files() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let initial = StagingDirectory::create(&fixture.0).unwrap();
    let outside = fixture.0.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("plugin.dll"), b"foreign").unwrap();
    let link = fixture.0.join(PARENT).join(uuid::Uuid::new_v4().to_string());
    symlink(&outside, &link).unwrap();
    let mut stale = StagingDirectory::create(&fixture.0).unwrap();
    let stale_path = stale.path().to_path_buf();
    symlink(outside.join("plugin.dll"), stale_path.join("plugin.dll")).unwrap();
    drop(stale.owner.take());
    let next = StagingDirectory::create(&fixture.0).unwrap();
    assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
    assert!(stale_path.is_dir());
    assert_eq!(fs::read(outside.join("plugin.dll")).unwrap(), b"foreign");
    drop(stale);
    drop(next);
    drop(initial);
}

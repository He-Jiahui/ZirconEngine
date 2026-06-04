use super::super::SoundBackendStatus;

pub trait SoundBackendManager {
    fn backend_name(&self) -> String;
    fn backend_status(&self) -> SoundBackendStatus;
}

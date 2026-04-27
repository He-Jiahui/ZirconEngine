#[derive(Clone, Debug, Default)]
pub struct PhysicsConfig {
    pub enabled: bool,
    pub backend: &'static str,
}

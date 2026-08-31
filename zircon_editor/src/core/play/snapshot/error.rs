use thiserror::Error;
use zircon_runtime::scene::DynamicSceneError;

#[derive(Clone, Debug, Error)]
pub enum PlaySceneSourceError {
    #[error("failed to capture the authoring world as a play scene: {0}")]
    DynamicScene(#[from] DynamicSceneError),
}

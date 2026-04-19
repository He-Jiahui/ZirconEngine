use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphicsError {
    #[error("wgpu surface acquisition status: {0}")]
    SurfaceStatus(&'static str),
    #[error("surface creation failed: {0}")]
    SurfaceCreation(#[from] wgpu::CreateSurfaceError),
    #[error("no compatible adapter found")]
    NoAdapter,
    #[error("request device failed: {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
    #[error("asset channel failure: {0}")]
    Channel(String),
    #[error("asset loading failed: {0}")]
    Asset(String),
    #[error("thread bootstrap failure: {0}")]
    ThreadBootstrap(String),
    #[error("buffer map failed: {0}")]
    BufferMap(String),
}

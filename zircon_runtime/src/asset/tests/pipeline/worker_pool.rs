use crate::asset::pipeline::types::{AssetRequest, CpuAssetPayload, TextureSource};
use crate::asset::pipeline::worker_pool::AssetWorkerPool;

#[test]
fn worker_pool_completes_builtin_texture_requests() {
    let pool = AssetWorkerPool::new(1).unwrap();
    let completions = pool.completion_receiver();

    pool.request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();

    let payload = completions.recv().unwrap();
    match payload {
        CpuAssetPayload::Texture(texture) => {
            assert_eq!(texture.source, TextureSource::BuiltinChecker);
            assert_eq!(
                texture.rgba.len(),
                texture.width as usize * texture.height as usize * 4
            );
        }
        other => panic!("unexpected payload: {other:?}"),
    }
}

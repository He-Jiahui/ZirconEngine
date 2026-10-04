use std::error::Error;
use std::io;

use zircon_runtime::scene::world::SceneProjectError;

use super::EditorError;

#[test]
fn scene_project_conversion_preserves_the_typed_source_chain() {
    let error: EditorError =
        SceneProjectError::from(io::Error::other("layout preset source")).into();
    let source = error
        .source()
        .expect("EditorError should expose its source");

    assert!(source.downcast_ref::<SceneProjectError>().is_some());
    assert!(source.source().is_some());
}

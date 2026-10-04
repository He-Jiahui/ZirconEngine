use crate::text::shaping::backend_error::BackendShapeError;
use crate::text::TextRange;

use super::DirectShapeError;

#[test]
fn backend_failures_retain_the_itemized_source_range() {
    let range = TextRange { start: 4, end: 9 };
    let face = crate::text::FontFaceId(7);

    let error = DirectShapeError::backend(
        range,
        BackendShapeError::FaceParseFailed {
            face,
            face_index: 2,
        },
    );

    assert!(matches!(
        error,
        DirectShapeError::Backend {
            range: retained,
            source: BackendShapeError::FaceParseFailed {
                face: retained_face,
                face_index: 2,
            },
        } if retained == range && retained_face == face
    ));
}

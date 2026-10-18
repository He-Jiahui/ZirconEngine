//! 直接错误封装的字段保留契约。
//!
//! 人工构造错误，只校验诊断字段的保留，不调用字体后端或证明完整成形路径。

use crate::text::shaping::backend_error::BackendShapeError;
use crate::text::TextRange;

use super::DirectShapeError;

/// 校验后端变体及 range、face、face_index 的保留；不检查 Display 或错误链接口。
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

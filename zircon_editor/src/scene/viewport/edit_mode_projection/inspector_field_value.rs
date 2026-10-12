use zircon_runtime::scene::EntityId;
use zircon_runtime_interface::math::Real;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SceneInspectorFieldValue {
    Bool(bool),
    Unsigned(u64),
    Scalar(Real),
    Vec2([Real; 2]),
    Vec3([Real; 3]),
    Vec4([Real; 4]),
    Quaternion([Real; 4]),
    Text(String),
    Entity(Option<EntityId>),
    Resource(String),
    Enum(String),
}

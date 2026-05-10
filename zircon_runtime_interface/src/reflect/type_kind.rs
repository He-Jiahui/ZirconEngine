use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReflectTypeKind {
    Struct,
    TupleStruct,
    Tuple,
    Enum,
    List,
    Map,
    Scalar,
    Opaque,
    Json,
}

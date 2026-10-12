#[path = "property_path/compile.rs"]
mod compile;
#[path = "property_path/model.rs"]
mod model;
#[path = "property_path/read.rs"]
mod read;
#[path = "property_path/write.rs"]
mod write;

pub use model::{
    CompiledScenePropertyTarget, CompiledScenePropertyWriter, ComponentFieldId, PathId,
};

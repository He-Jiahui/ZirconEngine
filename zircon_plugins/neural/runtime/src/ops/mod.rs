mod attrs;
mod op_code;

pub use attrs::{NnConv2dAttrs, NnGemmAttrs, NnOpAttrs, NnOpAttrsError, NnPool2dAttrs};
pub use op_code::NnOpCode;

#[derive(Clone, Debug, PartialEq)]
pub struct NnOp {
    pub code: NnOpCode,
    pub inputs: Vec<u16>,
    pub outputs: Vec<u16>,
    pub attrs: NnOpAttrs,
}

impl NnOp {
    pub fn new(code: NnOpCode, inputs: Vec<u16>, outputs: Vec<u16>, attrs: NnOpAttrs) -> Self {
        Self {
            code,
            inputs,
            outputs,
            attrs,
        }
    }
}

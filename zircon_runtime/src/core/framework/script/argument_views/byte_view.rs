use super::super::ScriptHostError;

pub trait ScriptHostByteSource {
    fn len(&self) -> Result<usize, ScriptHostError>;

    fn byte_at(&self, index: usize) -> Result<u8, ScriptHostError>;
}

#[derive(Clone, Copy)]
pub enum ScriptHostByteView<'call> {
    Slice(&'call [u8]),
    Source(&'call dyn ScriptHostByteSource),
}

impl ScriptHostByteView<'_> {
    pub fn len(&self) -> Result<usize, ScriptHostError> {
        match self {
            Self::Slice(bytes) => Ok(bytes.len()),
            Self::Source(source) => source.len(),
        }
    }

    pub fn byte_at(&self, index: usize) -> Result<u8, ScriptHostError> {
        match self {
            Self::Slice(bytes) => bytes.get(index).copied().ok_or_else(|| {
                ScriptHostError::new(format!("byte argument index {index} was not provided"))
            }),
            Self::Source(source) => source.byte_at(index),
        }
    }

    pub fn copy_to_vec_at_business_boundary(&self) -> Result<Vec<u8>, ScriptHostError> {
        let length = self.len()?;
        let mut bytes = Vec::with_capacity(length);
        for index in 0..length {
            bytes.push(self.byte_at(index)?);
        }
        Ok(bytes)
    }
}

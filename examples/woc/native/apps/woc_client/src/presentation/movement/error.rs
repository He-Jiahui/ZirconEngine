use woc_protocol::MovementInputError;

#[derive(Debug, PartialEq)]
pub enum ClientMovementInputError {
    InvalidInput(MovementInputError),
    SequenceExhausted,
}

impl From<MovementInputError> for ClientMovementInputError {
    fn from(error: MovementInputError) -> Self {
        Self::InvalidInput(error)
    }
}

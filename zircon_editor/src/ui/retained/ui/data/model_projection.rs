use crate::ui::retained_host::primitives::ModelRc;

pub(super) fn map_model_rc<T, U, F>(model: &ModelRc<T>, mut map: F) -> ModelRc<U>
where
    F: FnMut(&T) -> U,
{
    model.map_preserving_metadata(&mut map)
}

#[cfg(test)]
#[path = "tests/model_projection.rs"]
mod tests;

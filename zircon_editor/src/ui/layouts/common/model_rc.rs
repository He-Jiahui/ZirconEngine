use std::rc::Rc;

use slint::{ModelRc, VecModel};

pub(crate) fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};

pub(super) fn to_host_contract_shared_string_list(items: Vec<String>) -> ModelRc<SharedString> {
    model_rc(items.into_iter().map(SharedString::from).collect())
}

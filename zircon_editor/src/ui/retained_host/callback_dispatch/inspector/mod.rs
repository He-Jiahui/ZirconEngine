#[cfg(test)]
mod apply;
#[cfg(test)]
mod delete_selected;
#[cfg(test)]
mod draft_field;
mod surface_control;

#[cfg(test)]
pub(crate) use apply::dispatch_inspector_apply;
#[cfg(test)]
pub(crate) use delete_selected::dispatch_inspector_delete_selected;
#[cfg(test)]
pub(crate) use draft_field::dispatch_inspector_draft_field;
pub(crate) use surface_control::dispatch_builtin_inspector_surface_control;

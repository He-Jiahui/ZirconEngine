mod backend;
mod projection;

#[cfg(test)]
mod tests;

pub(super) use projection::apply_horizontal_backend_shaping;

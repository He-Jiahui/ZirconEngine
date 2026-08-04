#[macro_export]
macro_rules! tr {
    ($service:expr, $key:expr) => {
        $service.translate($key)
    };
}

/// Authority that contributed a value to the resolved product-host configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductConfigSource {
    ProductRole,
    RuntimeProfile,
    EntryRequest,
    ExportProfile,
}

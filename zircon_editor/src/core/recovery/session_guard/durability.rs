/// Whether the latest lock mutation was published and whether its directory sync was uncertain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionLockDurability {
    Published,
    PublishedWithDurabilityUncertainty,
}

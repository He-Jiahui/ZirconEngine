/// Reserved M2 service identity. M0 intentionally provides no constructible
/// driver until a real provider route, queue, cancellation, and shutdown owner
/// are connected to asset reads.
#[derive(Clone, Debug)]
pub enum AssetIoDriver {}

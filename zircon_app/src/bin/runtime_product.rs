// The shipping Runtime is a distinct Cargo target so its feature gate stays
// independent from the development preview target.  Keep the process contract
// in one source body to prevent those two hosts from drifting.
include!("runtime_preview.rs");

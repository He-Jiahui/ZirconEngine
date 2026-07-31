#![forbid(unsafe_code)]

mod generated_trace_symbols;
mod golden;
mod rng;
mod trace;
mod wire;

pub use generated_trace_symbols::{
    resolve_trace_symbol, trace_symbol_id, TRACE_SYMBOL_FINGERPRINT,
};
pub use golden::{GoldenError, GoldenScenario, GoldenSuite, GoldenUpdateGuard};
pub use rng::{DrawDigest, Mulberry32};
pub use trace::{canonical, digest, fnv1a_hex, fnv1a_step_u32, round6, TraceValue, FNV_OFFSET};
pub use wire::{decode_vm_trace, VmTraceDecodeLimits, VmTraceWireError};

pub const EXPECTED_GOLDEN_SCENARIOS: usize = 54;

pub fn reference_identity() -> woc_protocol::WocReferenceIdentity {
    woc_protocol::REFERENCE_IDENTITY
}

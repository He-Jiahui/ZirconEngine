#[path = "argument_views/argument_source.rs"]
mod argument_source;
#[path = "argument_views/byte_view.rs"]
mod byte_view;
#[path = "argument_views/typed_conversion.rs"]
mod typed_conversion;
#[path = "argument_views/value_ref.rs"]
mod value_ref;

pub(crate) use argument_source::ScriptHostOwnedArgumentSource;
pub use argument_source::{ScriptHostArgumentSource, ScriptHostArguments};
pub use byte_view::{ScriptHostByteSource, ScriptHostByteView};
pub use typed_conversion::ScriptHostFromArgument;
pub use value_ref::ScriptHostValueRef;

#[cfg(test)]
#[path = "argument_views/tests.rs"]
mod tests;

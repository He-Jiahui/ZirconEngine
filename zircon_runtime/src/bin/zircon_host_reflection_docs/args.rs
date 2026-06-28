use std::ffi::OsString;
use std::path::PathBuf;

use super::error::{HostReflectionDocsError, HostReflectionDocsResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostReflectionDocsArgs {
    pub output: PathBuf,
}

pub fn parse(
    args: impl IntoIterator<Item = OsString>,
) -> HostReflectionDocsResult<HostReflectionDocsArgs> {
    let mut args = args.into_iter();
    let output = args
        .next()
        .ok_or_else(|| HostReflectionDocsError::Usage(usage("missing output path")))
        .map(PathBuf::from)?;
    if args.next().is_some() {
        return Err(HostReflectionDocsError::Usage(usage(
            "expected exactly one output path",
        )));
    }

    Ok(HostReflectionDocsArgs { output })
}

pub fn usage(message: &str) -> String {
    format!("{message}\nusage: zircon_host_reflection_docs <output-markdown-path>")
}

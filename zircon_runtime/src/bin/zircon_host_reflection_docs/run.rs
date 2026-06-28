use std::ffi::OsString;

use zircon_runtime::script::{
    builtin_host_module_descriptors, write_script_host_modules_markdown,
    ScriptHostInterfaceMarkdownOptions,
};

use super::args::parse;
use super::error::{HostReflectionDocsError, HostReflectionDocsResult};

pub fn run(args: impl IntoIterator<Item = OsString>) -> HostReflectionDocsResult<()> {
    let args = parse(args)?;
    let modules = builtin_host_module_descriptors()
        .map_err(|source| HostReflectionDocsError::CollectBuiltInHostModules { source })?;
    write_script_host_modules_markdown(
        &args.output,
        &modules,
        &ScriptHostInterfaceMarkdownOptions::default(),
    )
    .map_err(|source| HostReflectionDocsError::WriteHostInterfaceDocs {
        path: args.output,
        source,
    })
}

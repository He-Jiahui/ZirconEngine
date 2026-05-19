use std::ffi::OsString;
use std::path::PathBuf;

use zircon_runtime::script::{
    builtin_host_module_descriptors, write_script_host_modules_markdown,
    ScriptHostInterfaceMarkdownOptions,
};

fn main() {
    if let Err(error) = run(std::env::args_os().skip(1)) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), String> {
    let mut args = args.into_iter();
    let output = args
        .next()
        .ok_or_else(|| usage("missing output path"))
        .map(PathBuf::from)?;
    if args.next().is_some() {
        return Err(usage("expected exactly one output path"));
    }

    let modules = builtin_host_module_descriptors()
        .map_err(|error| format!("failed to collect built-in host modules: {error}"))?;
    write_script_host_modules_markdown(
        &output,
        &modules,
        &ScriptHostInterfaceMarkdownOptions::default(),
    )
    .map_err(|error| {
        format!(
            "failed to write host interface docs to {}: {error}",
            output.display()
        )
    })
}

fn usage(message: &str) -> String {
    format!("{message}\nusage: zircon_host_reflection_docs <output-markdown-path>")
}

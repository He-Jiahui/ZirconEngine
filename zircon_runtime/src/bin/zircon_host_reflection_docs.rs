#[path = "zircon_host_reflection_docs/args.rs"]
mod args;
#[path = "zircon_host_reflection_docs/error.rs"]
mod error;
#[path = "zircon_host_reflection_docs/run.rs"]
mod run;

fn main() {
    if let Err(error) = run::run(std::env::args_os().skip(1)) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

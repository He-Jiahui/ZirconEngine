mod args;
mod error;
mod run;

fn main() {
    if let Err(error) = run::run(std::env::args_os().skip(1)) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

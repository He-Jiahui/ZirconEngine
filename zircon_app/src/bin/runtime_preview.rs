fn main() -> Result<(), Box<dyn std::error::Error>> {
    zircon_app::EntryRunner::run_runtime_with_args(std::env::args().skip(1))
}

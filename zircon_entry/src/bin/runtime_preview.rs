fn main() -> Result<(), Box<dyn std::error::Error>> {
    zircon_entry::EntryRunner::run_runtime()
}

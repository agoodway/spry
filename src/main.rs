fn main() {
    if let Err(err) = spry::cli::run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

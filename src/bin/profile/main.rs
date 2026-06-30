mod movegen;
mod search;

fn main() {
    let mut args = std::env::args();
    let _bin = args.next();
    match args.next().as_deref() {
        Some("movegen") => movegen::run(args),
        Some("search") => search::run(args),
        Some("--help") | Some("-h") | None => print_help(),
        Some(command) => panic!("unknown profile command: {command}"),
    }
}

fn print_help() {
    println!("Usage: profile <command> [options]");
    println!();
    println!("Commands:");
    println!("  movegen   Run deterministic fixed-work move generation profiling");
    println!("  search    Run deterministic fixed-work search profiling");
}

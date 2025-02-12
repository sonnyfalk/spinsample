use clap::Parser;

#[derive(Parser, Debug)]
struct Options {
    /// The process to sample
    pid: u64,
}

fn main() {
    let options = Options::parse();

    println!("pid: {}", options.pid);
}

use clap::Parser;

mod sampler;
use sampler::*;

#[derive(Parser, Debug)]
struct Options {
    /// The process to sample
    pid: Pid,
}

fn main() {
    let options = Options::parse();
    let _process_sample = ProcessSample::profile(options.pid);

    println!("pid: {}", options.pid);
}

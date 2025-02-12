use clap::Parser;

mod sampler;

#[derive(Parser, Debug)]
struct Options {
    /// The process to sample
    pid: u64,
}

fn main() {
    let options = Options::parse();
    let _process_sample = sampler::ProcessSample::profile(options.pid);

    println!("pid: {}", options.pid);
}

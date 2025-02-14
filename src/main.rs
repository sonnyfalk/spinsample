use clap::Parser;
use std::process::ExitCode;

mod sampler;

#[derive(Parser, Debug)]
struct Options {
    /// The process to sample
    pid: sampler::Pid,
}

fn main() -> ExitCode {
    let options = Options::parse();

    match sampler::profile(options.pid) {
        Ok(process_sample) => {
            println!("{}", process_sample);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Failed to sample pid {} - {}", options.pid, error);
            ExitCode::FAILURE
        }
    }
}

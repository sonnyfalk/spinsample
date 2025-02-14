use clap::Parser;
use std::process::ExitCode;

mod sampler;
use sampler::*;

#[derive(Parser, Debug)]
struct Options {
    /// The process to sample
    pid: Pid,
}

fn main() -> ExitCode {
    let options = Options::parse();

    match profile(options.pid) {
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

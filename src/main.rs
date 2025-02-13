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

    match ProcessSample::profile(options.pid) {
        Ok(_process_sample) => {
            println!("Done");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Failed to sample pid {} - {}", options.pid, error);
            ExitCode::FAILURE
        }
    }
}

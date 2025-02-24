use clap::Parser;
use std::process::ExitCode;
use std::time::Duration;

mod cancel_status;
mod process_iterator;
mod sampler;
mod thread_iterator;

pub use sampler::Pid;
pub use thread_iterator::*;

#[derive(Parser, Debug)]
struct Options {
    /// The process pid or name to sample
    process: String,
    /// Duration in seconds, default is 10
    duration: Option<u64>,
    /// Sampling interval in milliseconds, default is 1
    interval: Option<u64>,
}

fn main() -> ExitCode {
    let options = Options::parse();
    let Some(pid) = Pid::from_str_radix(&options.process, 10)
        .ok()
        .or_else(|| pid_for_name(&options.process))
    else {
        eprintln!("No such process - {}", options.process);
        return ExitCode::FAILURE;
    };

    match sampler::profile(
        pid,
        Duration::from_secs(options.duration.unwrap_or(10)),
        Duration::from_millis(options.interval.unwrap_or(1)),
    ) {
        Ok(process_sample) => {
            println!("{}", process_sample);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Failed to sample pid {} - {}", pid, error);
            ExitCode::FAILURE
        }
    }
}

fn pid_for_name(name: &str) -> Option<Pid> {
    let processes = process_iterator::ProcessIterator::snapshot()?;

    let matches: Vec<(String, Pid)> = processes
        .filter(|(process_name, _)| process_name.contains(name))
        .collect();

    // TODO: Filter down further and let the user pick one if there are multiple matches.
    Some(matches.first()?.1)
}

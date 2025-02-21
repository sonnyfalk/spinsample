use clap::Parser;
use std::process::ExitCode;

mod process_iterator;
mod sampler;
pub use sampler::Pid;

#[derive(Parser, Debug)]
struct Options {
    /// The process pid or name to sample
    process: String,
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

    match sampler::profile(pid) {
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

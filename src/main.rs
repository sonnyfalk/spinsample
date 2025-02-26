use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use windows::Win32::System::SystemInformation::GetLocalTime;

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
            if let Ok(file_path) = output_to_tmp_file(&process_sample) {
                println!("Sample analysis written to file {}\n", file_path.display());
            }
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

fn output_to_tmp_file(process_sample: &sampler::ProcessSample) -> std::io::Result<PathBuf> {
    let file_path = output_file_path(process_sample);
    let mut tmp_file = std::fs::File::create_new(&file_path)?;
    write!(tmp_file, "{}", process_sample)?;

    Ok(file_path)
}

fn output_file_path(process_sample: &sampler::ProcessSample) -> PathBuf {
    let name = process_sample
        .process_info
        .path
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_string)
        .unwrap_or(process_sample.process_info.pid.to_string());

    let date_time = unsafe { GetLocalTime() };

    let mut tmp_file = std::env::temp_dir();
    tmp_file.push(format!(
        "{}_{:04}-{:02}-{:02}_{:02}{:02}{:02}.spinsample.txt",
        name,
        date_time.wYear,
        date_time.wMonth,
        date_time.wDay,
        date_time.wHour,
        date_time.wMinute,
        date_time.wSecond
    ));

    tmp_file
}

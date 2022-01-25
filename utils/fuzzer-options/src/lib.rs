///! todo top level docs
use clap::{AppSettings, Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;

use libafl::bolts::os::Cores;

/// helper function to go from a parsed cli string to a `Duration`
fn parse_timeout(src: &str) -> Duration {
    Duration::from_millis(src.parse::<u64>().unwrap())
}

#[derive(Parser, Debug)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
pub struct FuzzerOptions {
    #[clap(subcommand)]
    pub command: Commands,

    /// timeout for each target execution (milliseconds)
    #[clap(short = 't', long = "timeout", takes_value = true, default_value = "2000", parse(from_str = parse_timeout))]
    pub timeout: Duration,

    /// whether or not to print debug info
    #[clap(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// file to which all client output should be written
    #[clap(short = 's', long = "stdout")]
    pub stdout: Option<String>,

    /// trailing arguments (after "--") will be passed directly to QEMU
    #[cfg(feature = "libafl_qemu")]
    #[clap(last = true)]
    pub qemu_args: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Fuzz mode: mutates the starting corpus indefinitely, looking for crashes
    Fuzz {
        #[clap(
            short = 'x',
            long = "tokens",
            multiple_values = true,
            parse(from_os_str)
        )]
        token_files: Vec<PathBuf>,

        /// input corpus directories
        #[clap(
            short = 'i',
            long = "corpus",
            default_values = &["corpus"],
            multiple_values = true,
            parse(from_os_str)
        )]
        corpora: Vec<PathBuf>,

        /// output solutions directory
        #[clap(
            short = 'o',
            long = "crashes",
            default_value = "solutions",
            parse(from_os_str)
        )]
        crashes: PathBuf,

        /// which cores to bind, i.e. --cores 1,2-4,5
        #[clap(short = 'c', long = "cores", default_value = "0", parse(try_from_str = Cores::from_cmdline))]
        cores: Cores,

        /// port on which the broker should listen
        #[clap(short = 'p', long = "port", default_value = "1337")]
        port: u16,
    },

    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    /// Replay mode: runs a single input file through the fuzz harness
    Replay {
        /// input corpus directories
        #[clap(short = 'i', long = "input-file", parse(from_os_str))]
        input_file: PathBuf,
    },
}

#[must_use]
/// Parse from `std::env::args_os()`, exit on error
pub fn parse_args() -> FuzzerOptions {
    FuzzerOptions::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "libafl_qemu")]
    /// pass a standard option and `--` followed by some options that FuzzerOptions doesn't know
    /// about; expect the standard option to work normally, and everything after `--` to be
    /// collected into `qemu_args`
    fn standard_option_with_trailing_variable_length_args_collected() {
        let parsed =
            FuzzerOptions::parse_from(["some-command", "--port", "1336", "--", "-L", "qemu-bound"]);
        assert_eq!(parsed.port, 1336);
        assert_eq!(parsed.qemu_args, ["-L", "qemu-bound"])
    }
}

use std::process::ExitCode;

mod cli;
mod commands;

fn main() -> ExitCode {
    match cli::parse() {
        Ok(cmd) => match commands::dispatch(cmd) {
            Ok(()) => ExitCode::from(0),
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(e.exit_code())
            }
        },
        Err(cli::CliError::Help) => {
            print_help();
            ExitCode::from(0)
        }
        Err(cli::CliError::Version) => {
            println!("rompatch {}", env!("CARGO_PKG_VERSION"));
            ExitCode::from(0)
        }
        Err(e) => {
            eprintln!("error: {e}");
            print_help();
            ExitCode::from(1)
        }
    }
}

fn print_help() {
    eprintln!(
        "rompatch - apply ROM patch files

USAGE:
    rompatch <COMMAND> [args]

COMMANDS:
    apply <ROM> <PATCH> [-o OUT]    Apply PATCH to ROM, write to OUT
    detect <PATCH>                   Print the detected patch format

OPTIONS:
    -h, --help                       Print help
    -V, --version                    Print version"
    );
}

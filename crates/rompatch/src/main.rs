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
    apply <ROM> <PATCH> [OPTIONS]    Apply PATCH to ROM
    detect <PATCH>                    Print the detected patch format
    info <PATCH>                      Print patch header metadata
    hash <FILE> [--algo <ALGO>]       Print file hash (default crc32)

APPLY OPTIONS:
    -o, --output <PATH>               Output path (default <rom>.patched.<ext>)
        --format <NAME>               Override autodetect (ips/ups/bps/pmsr/
                                      aps-gba/aps-n64/ppf/rup/bdf)
        --strip-header                Detect+strip an SMC/iNES/FDS/LYNX header
                                      before patching; reattach on write
        --fix-checksum                Recompute Game Boy or Mega Drive header
                                      checksum after patching
        --verify-input  <ALGO:HEX>    Check input ROM hash before patching
        --verify-output <ALGO:HEX>    Check output ROM hash after patching

HASH ALGORITHMS:
    crc32, md5, sha1, adler32

GENERAL OPTIONS:
    -h, --help                        Print help
    -V, --version                     Print version

SUPPORTED FORMATS:
    IPS, UPS, BPS, PMSR, APS (GBA + N64), PPF (v1/v2/v3), RUP, BDF"
    );
}

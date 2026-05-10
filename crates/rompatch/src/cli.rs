use core::fmt;
use std::path::PathBuf;

use lexopt::prelude::{Long, Short, Value};

#[derive(Debug)]
pub enum Command {
    Apply {
        rom: PathBuf,
        patch: PathBuf,
        out: Option<PathBuf>,
        strip_header: bool,
        fix_checksum: bool,
        verify_input: Option<String>,
        verify_output: Option<String>,
        format_override: Option<String>,
    },
    Detect {
        patch: PathBuf,
    },
    Info {
        patch: PathBuf,
    },
    Hash {
        file: PathBuf,
        algo: String,
    },
}

#[derive(Debug)]
pub enum CliError {
    Lexopt(lexopt::Error),
    UnknownSubcommand(String),
    MissingPositional(&'static str),
    Help,
    Version,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lexopt(e) => write!(f, "{e}"),
            Self::UnknownSubcommand(s) => write!(f, "unknown subcommand: {s}"),
            Self::MissingPositional(name) => write!(f, "missing required argument: <{name}>"),
            Self::Help => f.write_str("help requested"),
            Self::Version => f.write_str("version requested"),
        }
    }
}

impl From<lexopt::Error> for CliError {
    fn from(e: lexopt::Error) -> Self {
        Self::Lexopt(e)
    }
}

pub fn parse() -> Result<Command, CliError> {
    let mut parser = lexopt::Parser::from_env();
    let sub = match parser.next()? {
        Some(Long("help") | Short('h')) | None => return Err(CliError::Help),
        Some(Long("version") | Short('V')) => return Err(CliError::Version),
        Some(Value(v)) => v.into_string().map_err(lexopt::Error::NonUnicodeValue)?,
        Some(other) => return Err(other.unexpected().into()),
    };

    match sub.as_str() {
        "apply" => parse_apply(parser),
        "detect" => parse_detect(parser),
        "info" => parse_info(parser),
        "hash" => parse_hash(parser),
        other => Err(CliError::UnknownSubcommand(other.to_string())),
    }
}

fn parse_apply(mut p: lexopt::Parser) -> Result<Command, CliError> {
    let mut rom: Option<PathBuf> = None;
    let mut patch: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut strip_header = false;
    let mut fix_checksum = false;
    let mut verify_input: Option<String> = None;
    let mut verify_output: Option<String> = None;
    let mut format_override: Option<String> = None;
    while let Some(arg) = p.next()? {
        match arg {
            Short('o') | Long("output") => out = Some(PathBuf::from(p.value()?)),
            Long("strip-header") => strip_header = true,
            Long("fix-checksum") => fix_checksum = true,
            Long("verify-input") => {
                verify_input = Some(
                    p.value()?
                        .into_string()
                        .map_err(lexopt::Error::NonUnicodeValue)?,
                );
            }
            Long("verify-output") => {
                verify_output = Some(
                    p.value()?
                        .into_string()
                        .map_err(lexopt::Error::NonUnicodeValue)?,
                );
            }
            Long("format") => {
                format_override = Some(
                    p.value()?
                        .into_string()
                        .map_err(lexopt::Error::NonUnicodeValue)?,
                );
            }
            Long("help") | Short('h') => return Err(CliError::Help),
            Value(v) if rom.is_none() => rom = Some(PathBuf::from(v)),
            Value(v) if patch.is_none() => patch = Some(PathBuf::from(v)),
            other => return Err(other.unexpected().into()),
        }
    }
    Ok(Command::Apply {
        rom: rom.ok_or(CliError::MissingPositional("rom"))?,
        patch: patch.ok_or(CliError::MissingPositional("patch"))?,
        out,
        strip_header,
        fix_checksum,
        verify_input,
        verify_output,
        format_override,
    })
}

fn parse_detect(mut p: lexopt::Parser) -> Result<Command, CliError> {
    let patch = match p.next()? {
        Some(Value(v)) => PathBuf::from(v),
        Some(Long("help") | Short('h')) => return Err(CliError::Help),
        Some(other) => return Err(other.unexpected().into()),
        None => return Err(CliError::MissingPositional("patch")),
    };
    Ok(Command::Detect { patch })
}

fn parse_info(mut p: lexopt::Parser) -> Result<Command, CliError> {
    let patch = match p.next()? {
        Some(Value(v)) => PathBuf::from(v),
        Some(Long("help") | Short('h')) => return Err(CliError::Help),
        Some(other) => return Err(other.unexpected().into()),
        None => return Err(CliError::MissingPositional("patch")),
    };
    Ok(Command::Info { patch })
}

fn parse_hash(mut p: lexopt::Parser) -> Result<Command, CliError> {
    let mut file: Option<PathBuf> = None;
    let mut algo: Option<String> = None;
    while let Some(arg) = p.next()? {
        match arg {
            Long("algo") => {
                algo = Some(
                    p.value()?
                        .into_string()
                        .map_err(lexopt::Error::NonUnicodeValue)?,
                );
            }
            Long("help") | Short('h') => return Err(CliError::Help),
            Value(v) if file.is_none() => file = Some(PathBuf::from(v)),
            other => return Err(other.unexpected().into()),
        }
    }
    Ok(Command::Hash {
        file: file.ok_or(CliError::MissingPositional("file"))?,
        algo: algo.unwrap_or_else(|| "crc32".to_string()),
    })
}

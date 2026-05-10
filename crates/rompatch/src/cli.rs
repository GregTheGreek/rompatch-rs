use core::fmt;
use std::path::PathBuf;

use lexopt::prelude::{Long, Short, Value};

#[derive(Debug)]
pub enum Command {
    Apply {
        rom: PathBuf,
        patch: PathBuf,
        out: Option<PathBuf>,
    },
    Detect {
        patch: PathBuf,
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
        other => Err(CliError::UnknownSubcommand(other.to_string())),
    }
}

fn parse_apply(mut p: lexopt::Parser) -> Result<Command, CliError> {
    let mut rom: Option<PathBuf> = None;
    let mut patch: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    while let Some(arg) = p.next()? {
        match arg {
            Short('o') | Long("output") => out = Some(PathBuf::from(p.value()?)),
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

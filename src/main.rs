/*!
These are some docs.
*/

#![allow(dead_code, unused_variables)]

#![feature(collections, core, env, old_io, os, old_path, std_misc, unicode)]

extern crate csv;
extern crate docopt;
extern crate rand;
extern crate regex;
extern crate "rustc-serialize" as rustc_serialize;
extern crate stats;
extern crate tabwriter;

use std::borrow::ToOwned;
use std::error::FromError;
use std::env;
use std::fmt;
use std::old_io as io;

use docopt::Docopt;

macro_rules! werr {
    ($($arg:tt)*) => (
        match ::std::old_io::stderr().write_str(&*format!($($arg)*)) {
            Ok(_) => (),
            Err(err) => panic!("{}", err),
        }
    );
}

macro_rules! fail {
    ($e:expr) => (Err(::std::error::FromError::from_error($e)));
}

macro_rules! command_list {
    () => (
"
    cat         Concatenate by row or column
    count       Count records
    fixlengths  Makes all records have same length
    flatten     Show one field per line
    fmt         Format CSV output (change field delimiter)
    frequency   Show frequency tables
    headers     Show header names
    index       Create CSV index for faster access
    join        Join CSV files
    sample      Randomly sample CSV data
    search      Search CSV data with regexes
    select      Select columns from CSV
    slice       Slice records from CSV
    sort        Sort CSV data
    split       Split CSV data into many files
    stats       Compute basic statistics
    table       Align CSV data into columns
"
    )
}

static USAGE: &'static str = concat!("
Usage:
    xsv <command> [<args>...]
    xsv [options]

Options:
    --list        List all commands available.
    -h, --help    Display this message
    --version     Print version info and exit

Commands:", command_list!());

#[derive(RustcDecodable)]
struct Args {
    arg_command: Option<Command>,
    flag_list: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.options_first(true)
                                           .version(Some(util::version()))
                                           .decode())
                            .unwrap_or_else(|e| e.exit());
    if args.flag_list {
        let msg = concat!("Installed commands:", command_list!());
        io::stdout().write_str(msg).unwrap();
        return;
    }
    match args.arg_command {
        None => {
            env::set_exit_status(0);
            let msg = concat!(
                "xsv is a suite of CSV command line utilities.

Please choose one of the following commands:",
                command_list!());
            io::stderr().write_str(msg).unwrap();
        }
        Some(cmd) => {
            match cmd.run() {
                Ok(()) => env::set_exit_status(0),
                Err(CliError::Flag(err)) => err.exit(),
                Err(CliError::Csv(err)) => {
                    env::set_exit_status(1);
                    io::stderr()
                       .write_str(&*format!("{}\n", err.to_string()))
                       .unwrap();
                }
                Err(CliError::Io(
                        io::IoError { kind: io::BrokenPipe, .. })) => {
                    env::set_exit_status(0);
                }
                Err(CliError::Io(err)) => {
                    env::set_exit_status(1);
                    io::stderr()
                       .write_str(&*format!("{}\n", err.to_string()))
                       .unwrap();
                }
                Err(CliError::Other(msg)) => {
                    env::set_exit_status(1);
                    io::stderr()
                       .write_str(&*format!("{}\n", msg))
                       .unwrap();
                }
            }
        }
    }
}

#[derive(Debug, RustcDecodable)]
enum Command {
    Cat,
    Count,
    FixLengths,
    Flatten,
    Fmt,
    Frequency,
    Headers,
    Index,
    Join,
    Sample,
    Search,
    Select,
    Slice,
    Sort,
    Split,
    Stats,
    Table,
}

impl Command {
    fn run(self) -> CliResult<()> {
        let argv: Vec<_> = env::args().map(|v| v.to_string()).collect();
        let argv: Vec<_> = argv.iter().map(|s| &**s).collect();
        let argv = &*argv;
        match self {
            Command::Cat => cmd::cat::run(argv),
            Command::Count => cmd::count::run(argv),
            Command::FixLengths => cmd::fixlengths::run(argv),
            Command::Flatten => cmd::flatten::run(argv),
            Command::Fmt => cmd::fmt::run(argv),
            Command::Frequency => cmd::frequency::run(argv),
            Command::Headers => cmd::headers::run(argv),
            Command::Index => cmd::index::run(argv),
            Command::Join => cmd::join::run(argv),
            Command::Sample => cmd::sample::run(argv),
            Command::Search => cmd::search::run(argv),
            Command::Select => cmd::select::run(argv),
            Command::Slice => cmd::slice::run(argv),
            Command::Sort => cmd::sort::run(argv),
            Command::Split => cmd::split::run(argv),
            Command::Stats => cmd::stats::run(argv),
            Command::Table => cmd::table::run(argv),
        }
    }
}

type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
enum CliError {
    Flag(docopt::Error),
    Csv(csv::Error),
    Io(io::IoError),
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Flag(ref e) => { e.fmt(f) }
            CliError::Csv(ref e) => { e.fmt(f) }
            CliError::Io(ref e) => { e.fmt(f) }
            CliError::Other(ref s) => { f.write_str(&**s) }
        }
    }
}

impl FromError<docopt::Error> for CliError {
    fn from_error(err: docopt::Error) -> CliError {
        CliError::Flag(err)
    }
}

impl FromError<csv::Error> for CliError {
    fn from_error(err: csv::Error) -> CliError {
        match err {
            csv::Error::Io(v) => FromError::from_error(v),
            v => CliError::Csv(v),
        }
    }
}

impl FromError<io::IoError> for CliError {
    fn from_error(err: io::IoError) -> CliError {
        CliError::Io(err)
    }
}

impl FromError<String> for CliError {
    fn from_error(err: String) -> CliError {
        CliError::Other(err)
    }
}

impl<'a> FromError<&'a str> for CliError {
    fn from_error(err: &'a str) -> CliError {
        CliError::Other(err.to_owned())
    }
}

impl FromError<regex::Error> for CliError {
    fn from_error(err: regex::Error) -> CliError {
        CliError::Other(format!("{:?}", err))
    }
}

mod cmd;
mod config;
mod select;
mod util;

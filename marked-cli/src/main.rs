#![warn(rust_2018_idioms)]

use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::process;
use std::fs::File;

use encoding_rs as enc;
// use html5ever::driver::ParseOpts;
// use html5ever::tree_builder::TreeBuilderOpts;

use marked::EncodingHint;
use marked::html::parse_buffered;

use marked::logger::setup_logger;

use clap::{
    crate_version,
    Arg, App, AppSettings, SubCommand,
};

use log::error;

// Conveniently compact type alias for dyn Trait `std::error::Error`.
type Flaw = Box<dyn StdError + Send + Sync + 'static>;

#[derive(Debug)]
pub(crate) struct ClError(String);

impl fmt::Display for ClError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl StdError for ClError {}

macro_rules! quit {
    ($($args:tt)+) => {
        return Err($crate::ClError(format!($($args)+)).into())
    };
}

fn main() {
    let r = run();
    if let Err(e) = r {
        error!("{}", e);
        process::exit(2);
    }
}

fn run() -> Result<(), Flaw> {
    let html = SubCommand::with_name("html")
        .setting(AppSettings::DeriveDisplayOrder)
        .about("HTML processing")
        .after_help(
            "Parses input, applies filters, and serializes to output.")
        .args(&[
            Arg::with_name("output")
                .short("o")
                //.long("output")
                .number_of_values(1)
                .help("Output to specified file (default: STDOUT)"),
            Arg::with_name("file")
                .required(false)
                .value_name("INPUT-FILE")
                .help("File path to read (default: STDIN)")
        ]);

    let app = App::new("marked")
        .version(crate_version!())
        .about("Tool for *ML I/O filtering")
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::DeriveDisplayOrder)
        .max_term_width(100)
        .arg(Arg::with_name("debug")
             .short("d")
             .long("debug")
             .multiple(true)
             .help("Enable more logging, and up to `-dddd`")
             .global(true))
        .subcommand(html);

    let m = app.get_matches();
    setup_logger(m.occurrences_of("debug") as u32)?;

    /*
    let _opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    }; // FIXME: allow passing to parse_buffered?
     */

    let scname = m.subcommand_name().unwrap(); // required
    if scname != "html" {
        quit!("only html (command) processing is supported")
    }
    let subm = m.subcommand_matches(scname).unwrap();

    let eh = EncodingHint::shared_default(enc::UTF_8);
    let fin = subm.value_of("file");
    let mut input: Box<dyn io::Read> = if let Some(fin) = fin {
        Box::new(File::open(fin)?)
    } else {
        Box::new(io::stdin())
    };

    let doc = parse_buffered(eh, &mut input)?;

    // check dom.errors?

    let mut output: Box<dyn io::Write> =
        if let Some(fout) = subm.value_of("output")
    {
        if Some(fout) != fin {
            Box::new(File::create(fout)?)
        } else {
            quit!(
                "input {} same as output {} not supported",
                fin.unwrap(), fout
            );
        }
    } else {
        Box::new(io::stdout())
    };

    doc.serialize(&mut output)?;

    Ok(())
}

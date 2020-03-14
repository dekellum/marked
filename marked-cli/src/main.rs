#![warn(rust_2018_idioms)]

use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::process;
use std::fs::File;

use encoding_rs as enc;
// use html5ever::driver::ParseOpts;
// use html5ever::tree_builder::TreeBuilderOpts;

use marked::{
    chain_filters,
    filter,
    html::parse_buffered,
    logger::setup_logger,
    EncodingHint,
};

use clap::{
    crate_version,
    Arg, App, AppSettings, SubCommand,
};

use log::{debug, error};

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
                .long("output")
                .number_of_values(1)
                .help("Output to specified file (default: STDOUT)"),
            Arg::with_name("encoding")
                .short("e")
                .long("encoding")
                .number_of_values(1)
                .multiple(true)
                .help("Hint at input encoding label (default: UTF-8)"),
            Arg::with_name("filter-banned")
                .short("f")
                .long("filter-banned")
                .help("Filter banned tags and attributes"),
            Arg::with_name("text-normalize")
                .short("t")
                .long("text-normalize")
                .help("Aggressively normalize document text"),
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

    let mtch = app.get_matches();
    setup_logger(mtch.occurrences_of("debug") as u32)?;

    /*
    let _opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    // FIXME: allow passing to parse_buffered?
    */

    let scname = mtch.subcommand_name().unwrap(); // required
    if scname != "html" {
        quit!("only html (command) processing is supported")
    }
    let mtch = mtch.subcommand_matches(scname).unwrap();

    let eh = EncodingHint::shared_default(enc::UTF_8);

    if let Some(vals) = mtch.values_of("encoding") {
        for enc in vals {
            eh.borrow_mut().add_label_hint(&enc, 0.11);
        }
        debug!("encoding hint {:?}", eh.borrow());
    }

    let fin = mtch.value_of("file");
    let mut input: Box<dyn io::Read> = if let Some(fin) = fin {
        Box::new(File::open(fin)?)
    } else {
        Box::new(io::stdin())
    };

    let mut doc = parse_buffered(eh, &mut input)?;

    // FIXME: report errors?

    if mtch.is_present("filter-banned") {
        doc.filter_breadth(chain_filters!(
            filter::detach_banned_elements,
            filter::detach_comments,
            filter::detach_pis,
            filter::retain_basic_attributes,
            filter::xmp_to_pre,
        ));
    }

    if mtch.is_present("text-normalize") {
        doc.filter(filter::fold_empty_inline);
        doc.filter(filter::text_normalize); // Always use new pass.
    }

    let fout = mtch.value_of("output");
    let mut output: Box<dyn io::Write> = if let Some(fout) = fout {
        if Some(fout) != fin {
            Box::new(File::create(fout)?)
        } else {
            quit!(
                "input {} same as output {} not supported",
                fin.unwrap(), fout);
        }
    } else {
        Box::new(io::stdout())
    };

    doc.serialize(&mut output)?;

    Ok(())
}

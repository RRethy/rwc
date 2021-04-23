use crate::format::{parse_format, Format};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "rwc", about = "Print counts of various things in <files>.")]
pub struct Cli {
    #[structopt(short, long, help = "Print byte counts.")]
    pub bytes: bool,

    #[structopt(short, long, help = "Print utf-8 character counts.")]
    pub chars: bool,

    #[structopt(
        short,
        long,
        help = "Print word counts. A word is a non-zero-length sequence of non-whitespace characters delimited by ascii whitespace."
    )]
    pub words: bool,

    #[structopt(short, long, help = "Print newline counts.")]
    pub lines: bool,

    #[structopt(long, help = "Include an extra row showing count totals.")]
    pub show_totals: bool,

    #[structopt(long, default_value = "table", parse(try_from_str = parse_format), help = "TODO")]
    pub format: Format,

    #[structopt(
        long,
        help = "Read input from the files specified by null separated paths in <files0_from>. If <files0_from> is - then read \\n separated paths from standard input."
    )]
    pub files0_from: Option<PathBuf>,

    #[structopt(help = "Files to read. If no paths are provided then read standard input.")]
    pub files: Vec<PathBuf>,
}

/// Just the opts passed from the command-line not including the paths. This is because we want
/// to use separate owners for the files named and the opts.
#[derive(Debug)]
pub struct Options {
    pub bytes: bool,
    pub chars: bool,
    pub words: bool,
    pub lines: bool,
    pub show_totals: bool,
}

impl From<&Cli> for Options {
    /// Sets up some default values
    fn from(cli: &Cli) -> Options {
        if !(cli.bytes || cli.chars || cli.words || cli.lines) {
            Options {
                bytes: true,
                chars: false,
                words: true,
                lines: true,
                show_totals: cli.show_totals,
            }
        } else {
            Options {
                bytes: cli.bytes,
                chars: cli.chars,
                words: cli.words,
                lines: cli.lines,
                show_totals: cli.show_totals,
            }
        }
    }
}

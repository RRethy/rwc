use rayon::prelude::*;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

mod cli;
mod count;
mod error;
mod format;
mod print;

use cli::{Cli, Options};
use count::{Countable, CountablePath, Counts};
use error::Error;
use format::Format;
use print::print;

/// Read and return null separated utf8 paths from readable
fn read_paths0_from<R: Read>(readable: R) -> Result<Vec<PathBuf>, Error> {
    let (fnames, errors): (Vec<_>, Vec<_>) = BufReader::new(readable)
        .split(b'\0')
        .partition(Result::is_ok);
    if errors.len() > 0 {
        return Err(errors
            .into_iter()
            .map(Result::unwrap_err)
            .map(Error::from)
            .collect::<Vec<Error>>()
            .into());
    }
    let (fnames, errors): (Vec<_>, Vec<_>) = fnames
        .into_iter()
        .map(Result::unwrap)
        .map(|fname| String::from_utf8(fname))
        .partition(Result::is_ok);
    if errors.len() > 0 {
        return Err(errors
            .into_iter()
            .map(Result::unwrap_err)
            .map(Error::from)
            .collect::<Vec<Error>>()
            .into());
    }
    Ok(fnames
        .into_iter()
        .map(Result::unwrap)
        .map(|fname| PathBuf::from(fname))
        .collect())
}

fn count_paths(paths: Vec<PathBuf>, opts: &Options) -> Vec<(Result<Counts, Error>, PathBuf)> {
    paths
        .into_par_iter()
        .map(|path| {
            let c = (&path).count(opts.bytes, opts.chars, opts.words, opts.lines);
            (c, path)
        })
        .collect()
}

fn run<R: Read, W: Write>(
    mut opts: Options,
    files0_from: Option<PathBuf>,
    files: Vec<PathBuf>,
    input: R,
    output: W,
    fmt: Format,
) -> Result<(), Error> {
    let mut counts = if let Some(from) = files0_from {
        if files.len() > 0 {
            return Err(String::from("file operands cannot be combined with --files0-from").into());
        }

        let paths = if *from == PathBuf::from("-") {
            // read null separated paths from stdin
            read_paths0_from(input)?
        } else {
            // read null separated paths from file
            match File::open(from) {
                Ok(f) => read_paths0_from(f)?,
                Err(e) => return Err(e.into()),
            }
        };
        count_paths(paths, &opts)
    } else if files.len() > 0 {
        count_paths(files, &opts)
    } else {
        opts.show_totals = true;
        vec![(
            input.count(opts.bytes, opts.chars, opts.words, opts.lines),
            PathBuf::from("Stdin"),
        )]
    };

    counts.par_sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or_else(|| Ordering::Less));

    print(fmt, counts, &opts, output)?;
    Ok(())
}

fn main() {
    let cli = Cli::from_args();
    let opts = Options::from(&cli);
    let files0_from = cli.files0_from;
    let files = cli.files;
    let fmt = cli.format;

    match run(opts, files0_from, files, io::stdin(), io::stdout(), fmt) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_opts() -> Options {
        Options {
            bytes: true,
            chars: false,
            words: true,
            lines: true,
            show_totals: false,
        }
    }

    #[test]
    fn test_run_default_arguments() {
        let cli = Cli {
            bytes: false,
            chars: false,
            words: false,
            lines: false,
            show_totals: false,
            format: format::Format::Table,
            files0_from: None,
            files: Vec::new(),
        };
        let opts = Options::from(&cli);
        assert!(opts.bytes);
        assert!(!opts.chars);
        assert!(opts.words);
        assert!(opts.lines);
        assert!(!opts.show_totals);
    }

    #[test]
    #[should_panic]
    fn test_run_cannot_combine_files0_from_and_files() {
        let files0_from = Some(PathBuf::new());
        let files = vec![PathBuf::new()];
        run(
            default_opts(),
            files0_from,
            files,
            io::stdin(),
            io::stdout(),
            Format::CSV,
        )
        .unwrap();
    }

    #[test]
    fn test_run_files0_from_stdin() {
        let files0_from = Some(PathBuf::from("-"));
        let stdin = b"test_data/default.txt\0test_data/ten_mb.txt";
        let mut stdout = Vec::new();
        run(
            default_opts(),
            files0_from,
            Vec::new(),
            &stdin[..],
            &mut stdout,
            Format::CSV,
        )
        .unwrap();
        assert_eq!(
            r"path,bytes,words,lines
test_data/default.txt,1048697,183155,20681
test_data/ten_mb.txt,10000000,2000000,1000000",
            String::from_utf8(stdout).unwrap()
        );
    }

    #[test]
    fn test_run_files0_from_paths() {
        let files0_from = Some(PathBuf::from("test_data/files0_from.txt"));
        let mut stdout = Vec::new();
        run(
            default_opts(),
            files0_from,
            Vec::new(),
            io::stdin(),
            &mut stdout,
            Format::CSV,
        )
        .unwrap();
        assert_eq!(
            r"path,bytes,words,lines
test_data/default.txt,1048697,183155,20681
test_data/ten_mb.txt,10000000,2000000,1000000",
            String::from_utf8(stdout).unwrap()
        );
    }

    #[test]
    fn test_run_files() {
        let mut stdout = Vec::new();
        run(
            default_opts(),
            None,
            vec![
                PathBuf::from("test_data/default.txt"),
                PathBuf::from("test_data/ten_mb.txt"),
            ],
            io::stdin(),
            &mut stdout,
            Format::CSV,
        )
        .unwrap();
        assert_eq!(
            r"path,bytes,words,lines
test_data/default.txt,1048697,183155,20681
test_data/ten_mb.txt,10000000,2000000,1000000",
            String::from_utf8(stdout).unwrap()
        );
    }

    #[test]
    fn test_run_stdin() {
        let stdin = b"this is some text\nthis is another line";
        let mut stdout = Vec::new();
        run(
            default_opts(),
            None,
            Vec::new(),
            &stdin[..],
            &mut stdout,
            Format::CSV,
        )
        .unwrap();
        assert_eq!(
            r"path,bytes,words,lines
Stdin,38,8,1
Totals,38,8,1",
            String::from_utf8(stdout).unwrap()
        );
    }
}

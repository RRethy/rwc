use crate::cli::Options;
use crate::count::Counts;
use crate::error::Error;
use crate::format::Format;
use colored::*;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use term_table::row::Row;
use term_table::table_cell::Alignment;
use term_table::table_cell::TableCell;
use term_table::{Table, TableStyle};

pub fn print<W: Write>(
    fmt: Format,
    results: Vec<(Result<Counts, Error>, PathBuf)>,
    opts: &Options,
    w: W,
) -> Result<(), Error> {
    match fmt {
        Format::Table => print_table(results, opts, w)?,
        Format::CSV => print_csv(results, opts, w)?,
    }
    Ok(())
}

fn print_table<W: Write>(
    results: Vec<(Result<Counts, Error>, PathBuf)>,
    opts: &Options,
    mut w: W,
) -> Result<(), Error> {
    let mut table = Table::new();
    table.style = TableStyle::rounded();

    fn make_cell<'a, T: fmt::Display>(data: &T) -> TableCell<'a> {
        TableCell::new_with_alignment_and_padding(data, 1, Alignment::Left, true)
    }

    let mut header = vec![make_cell(&"path".blue().bold())];
    if opts.bytes {
        header.push(make_cell(&"bytes".blue().bold()));
    }
    if opts.chars {
        header.push(make_cell(&"chars".blue().bold()));
    }
    if opts.words {
        header.push(make_cell(&"words".blue().bold()));
    }
    if opts.lines {
        header.push(make_cell(&"lines".blue().bold()));
    }
    table.add_row(Row::new(header));

    let mut total_bytes: usize = 0;
    let mut total_chars: usize = 0;
    let mut total_words: usize = 0;
    let mut total_lines: usize = 0;

    for pair in results {
        let (res, path) = pair;
        let mut cells = vec![make_cell(&path.display().to_string().green().bold())];
        match res {
            Ok(c) => {
                if opts.bytes {
                    cells.push(make_cell(&c.bytes));
                    total_bytes = total_bytes + c.bytes;
                }
                if opts.chars {
                    cells.push(make_cell(&c.chars));
                    total_chars = total_chars + c.chars;
                }
                if opts.words {
                    cells.push(make_cell(&c.words));
                    total_words = total_words + c.words;
                }
                if opts.lines {
                    cells.push(make_cell(&c.lines));
                    total_lines = total_lines + c.lines;
                }
            }
            Err(err) => {
                cells.push(TableCell::new_with_alignment_and_padding(
                    err,
                    table.rows[0].cells.len() - 1,
                    Alignment::Center,
                    false,
                ));
            }
        }
        table.add_row(Row::new(cells));
    }

    if opts.show_totals {
        let mut totals = vec![make_cell(&"Totals".magenta().bold())];
        if opts.bytes {
            totals.push(make_cell(&total_bytes));
        }
        if opts.chars {
            totals.push(make_cell(&total_chars));
        }
        if opts.words {
            totals.push(make_cell(&total_words));
        }
        if opts.lines {
            totals.push(make_cell(&total_lines));
        }
        table.add_row(Row::new(totals));
    }

    write!(w, "{}", table.render())?;
    Ok(())
}

fn print_csv<W: Write>(
    results: Vec<(Result<Counts, Error>, PathBuf)>,
    opts: &Options,
    mut w: W,
) -> Result<(), Error> {
    let mut rows = Vec::new();

    let mut header = vec!["path"];
    if opts.bytes {
        header.push("bytes");
    }
    if opts.chars {
        header.push("chars");
    }
    if opts.words {
        header.push("words");
    }
    if opts.lines {
        header.push("lines");
    }
    rows.push(header.join(","));

    let mut total_bytes: usize = 0;
    let mut total_chars: usize = 0;
    let mut total_words: usize = 0;
    let mut total_lines: usize = 0;

    for pair in results {
        let (res, path) = pair;
        let mut cells = vec![path.display().to_string()];
        match res {
            Ok(c) => {
                if opts.bytes {
                    cells.push(c.bytes.to_string());
                    total_bytes = total_bytes + c.bytes;
                }
                if opts.chars {
                    cells.push(c.chars.to_string());
                    total_chars = total_chars + c.chars;
                }
                if opts.words {
                    cells.push(c.words.to_string());
                    total_words = total_words + c.words;
                }
                if opts.lines {
                    cells.push(c.lines.to_string());
                    total_lines = total_lines + c.lines;
                }
            }
            Err(err) => {
                cells.push(err.to_string());
            }
        }
        rows.push(cells.join(","));
    }

    if opts.show_totals {
        let mut totals = vec![String::from("Totals")];
        if opts.bytes {
            totals.push(total_bytes.to_string());
        }
        if opts.chars {
            totals.push(total_chars.to_string());
        }
        if opts.words {
            totals.push(total_words.to_string());
        }
        if opts.lines {
            totals.push(total_lines.to_string());
        }
        rows.push(totals.join(","));
    }

    write!(w, "{}", rows.join("\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::count::Count;

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
    fn test_print_csv() {
        let results = vec![
            (
                Ok(Counts {
                    bytes: Count { val: Some(6) },
                    chars: Count { val: Some(7) },
                    words: Count { val: Some(8) },
                    lines: Count { val: Some(9) },
                }),
                PathBuf::from("foobar"),
            ),
            (
                Ok(Counts {
                    bytes: Count { val: Some(2) },
                    chars: Count { val: Some(3) },
                    words: Count { val: Some(4) },
                    lines: Count { val: Some(5) },
                }),
                PathBuf::from("baz"),
            ),
        ];
        let mut stdout = Vec::new();
        print_csv(results, &default_opts(), &mut stdout).unwrap();
        assert_eq!(
            r"path,bytes,words,lines
foobar,6,8,9
baz,2,4,5",
            String::from_utf8(stdout).unwrap()
        );
    }
}

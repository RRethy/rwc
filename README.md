# rwc

Rust implementation of wc with a slightly nicer UI.

The git history is gone because this was originally a minor component of another project. However, it was getting out of scope and didn't really belong so I'm putting the relevant code into its own repo.

## Installation

```
cargo install --git https://github.com/RRethy/rwc
```

## Usage

```
$ rwc --help

rwc 0.1.0
Print counts of various things in <files>.

USAGE:
    rwc [FLAGS] [OPTIONS] [files]...

FLAGS:
    -b, --bytes          Print byte counts.
    -c, --chars          Print utf-8 character counts.
    -h, --help           Prints help information
    -l, --lines          Print newline counts.
        --show-totals    Include an extra row showing count totals.
    -V, --version        Prints version information
    -w, --words          Print word counts. A word is a non-zero-length sequence of non-whitespace characters delimited
                         by ascii whitespace.

OPTIONS:
        --files0-from <files0-from>    Read input from the files specified by null separated paths in <files0_from>. If
                                       <files0_from> is - then read \n separated paths from standard input.
        --format <format>              TODO [default: table]

ARGS:
    <files>...    Files to read. If no paths are provided then read standard input.
```

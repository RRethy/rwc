use crate::error::Error;
use bytecount;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::ops;
use std::path::Path;
use utf8::BufReadDecoder;

const BUFFER_SIZE: usize = 1048576;

#[derive(Debug)]
pub struct Count {
    pub val: Option<usize>,
}

impl ops::Add<Count> for usize {
    type Output = usize;

    fn add(self, rhs: Count) -> usize {
        if let Some(n) = rhs.val {
            return self + n;
        }
        return self;
    }
}

impl fmt::Display for Count {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(n) = self.val {
            write!(f, "{}", n)
        } else {
            write!(f, "N/A")
        }
    }
}

#[derive(Debug)]
pub struct Counts {
    pub bytes: Count,
    pub chars: Count,
    pub words: Count,
    pub lines: Count,
}

pub trait Countable {
    fn count(self, bytes: bool, chars: bool, words: bool, lines: bool) -> Result<Counts, Error>;
}

pub trait CountablePath {
    fn count(self, bytes: bool, chars: bool, words: bool, lines: bool) -> Result<Counts, Error>;
}

impl<P: AsRef<Path>> CountablePath for P {
    fn count(self, bytes: bool, chars: bool, words: bool, lines: bool) -> Result<Counts, Error> {
        if bytes && !(chars || words || lines) {
            count_bytes(self)
        } else {
            count_readable(File::open(self)?, bytes, chars, words, lines)
        }
    }
}

impl<R: Read> Countable for R {
    fn count(self, bytes: bool, chars: bool, words: bool, lines: bool) -> Result<Counts, Error> {
        count_readable(self, bytes, chars, words, lines)
    }
}

fn count_readable<R: Read>(
    readable: R,
    _bytes: bool,
    chars: bool,
    words: bool,
    lines: bool,
) -> Result<Counts, Error> {
    let reader = BufReader::with_capacity(BUFFER_SIZE, readable);
    if chars {
        count_bytes_chars_words_lines(reader)
    } else if lines && !words {
        count_bytes_lines(reader)
    } else {
        count_bytes_words_lines(reader)
    }
}

pub(crate) fn count_bytes<P: AsRef<Path>>(path: P) -> Result<Counts, Error> {
    let bytes = fs::metadata(path)?.len() as usize;
    Ok(Counts {
        bytes: Count { val: Some(bytes) },
        chars: Count { val: None },
        words: Count { val: None },
        lines: Count { val: None },
    })
}

pub(crate) fn count_bytes_words_lines<T: Read>(mut reader: BufReader<T>) -> Result<Counts, Error> {
    let (mut bytes, mut words, mut lines) = (0, 0, 0);
    let mut in_word = false;
    loop {
        let buffer = reader.fill_buf()?;
        let len = buffer.len();
        if len == 0 {
            break;
        }
        bytes += len;
        for &b in buffer {
            lines += if b == b'\n' { 1 } else { 0 };
            if b.is_ascii_whitespace() {
                words += if in_word { 1 } else { 0 };
                in_word = false;
            } else {
                in_word = true;
            }
        }
        reader.consume(len);
    }
    if in_word {
        words += 1;
    }
    Ok(Counts {
        bytes: Count { val: Some(bytes) },
        chars: Count { val: None },
        words: Count { val: Some(words) },
        lines: Count { val: Some(lines) },
    })
}

pub(crate) fn count_bytes_chars_words_lines<T: Read>(
    reader: BufReader<T>,
) -> Result<Counts, Error> {
    let (mut bytes, mut chars, mut words, mut lines) = (0, 0, 0, 0);
    let mut in_word = false;
    let mut decoder = BufReadDecoder::new(reader);
    loop {
        if let Some(res) = decoder.next_strict() {
            let str = res?;
            bytes += str.len();
            for c in str.chars() {
                chars += 1;
                lines += if c == '\n' { 1 } else { 0 };
                if c.is_ascii_whitespace() {
                    words += if in_word { 1 } else { 0 };
                    in_word = false;
                } else {
                    in_word = true;
                }
            }
        } else {
            break;
        }
    }
    if in_word {
        words += 1;
    }
    Ok(Counts {
        bytes: Count { val: Some(bytes) },
        chars: Count { val: Some(chars) },
        words: Count { val: Some(words) },
        lines: Count { val: Some(lines) },
    })
}

pub(crate) fn count_bytes_lines<T: Read>(mut reader: BufReader<T>) -> Result<Counts, Error> {
    let (mut bytes, mut lines) = (0, 0);
    loop {
        let buffer = reader.fill_buf()?;
        let len = buffer.len();
        if len == 0 {
            break;
        }
        bytes += len;
        lines += bytecount::count(buffer, b'\n');
        reader.consume(len);
    }
    Ok(Counts {
        bytes: Count { val: Some(bytes) },
        chars: Count { val: None },
        words: Count { val: None },
        lines: Count { val: Some(lines) },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_count_bytes_words_lines() {
        let text: &[u8] =
            "hello😀😃😄😁😆😅😂🤣😀😃😄😁 hello world 12345\n67890😀 😃 😄 😁".as_bytes();
        let reader = BufReader::with_capacity(10, text);
        let counts = count_bytes_words_lines(reader).unwrap();
        assert_eq!(96, counts.bytes.val.unwrap(),);
        assert_eq!(8, counts.words.val.unwrap(),);
        assert_eq!(1, counts.lines.val.unwrap(),);

        let path: PathBuf = ["test_data", "default.txt"].iter().collect();
        let counts = path.count(true, false, true, true).unwrap();
        assert_eq!(1048697, counts.bytes.val.unwrap());
        assert_eq!(183155, counts.words.val.unwrap());
        assert_eq!(20681, counts.lines.val.unwrap());
    }

    #[test]
    fn test_count_bytes_chars_words_lines() {
        let text: &[u8] =
            "hello😀😃😄😁😆😅😂🤣😀😃😄😁 hello world 12345\n67890😀 😃 😄 😁".as_bytes();
        let reader = BufReader::with_capacity(10, text);
        let counts = count_bytes_chars_words_lines(reader).unwrap();
        assert_eq!(96, counts.bytes.val.unwrap(),);
        assert_eq!(48, counts.chars.val.unwrap(),);
        assert_eq!(8, counts.words.val.unwrap(),);
        assert_eq!(1, counts.lines.val.unwrap(),);

        let path: PathBuf = ["test_data", "default.txt"].iter().collect();
        let counts = path.count(true, true, true, true).unwrap();
        assert_eq!(1048697, counts.bytes.val.unwrap());
        assert_eq!(726780, counts.chars.val.unwrap());
        assert_eq!(183155, counts.words.val.unwrap());
        assert_eq!(20681, counts.lines.val.unwrap());
    }

    #[test]
    fn test_count_bytes_lines() {
        let text: &[u8] =
            "hello😀😃😄😁😆😅😂🤣😀😃😄😁 hello world 12345\n67890😀 😃 😄 😁".as_bytes();
        let reader = BufReader::with_capacity(10, text);
        let counts = count_bytes_lines(reader).unwrap();
        assert_eq!(96, counts.bytes.val.unwrap(),);
        assert_eq!(1, counts.lines.val.unwrap(),);

        let path: PathBuf = ["test_data", "default.txt"].iter().collect();
        let counts = path.count(true, false, false, true).unwrap();
        assert_eq!(20681, counts.lines.val.unwrap());
    }

    #[test]
    fn test_count_bytes() {
        let path: PathBuf = ["test_data", "default.txt"].iter().collect();
        let counts = path.count(true, false, false, false).unwrap();
        assert_eq!(counts.bytes.val.unwrap(), 1048697);
    }

    #[test]
    fn adding_counts() {
        let n = 1;
        let c = Count { val: Some(2) };
        assert_eq!((n + c), 3);

        let c = Count { val: None };
        assert_eq!((n + c), 1);
    }
}

use crate::error::Error;

pub fn parse_format(src: &str) -> Result<Format, Error> {
    match src {
        "table" => Ok(Format::Table),
        "csv" => Ok(Format::CSV),
        _ => Err(Error::PARSEFORMAT(src.into())),
    }
}

#[derive(Debug)]
pub enum Format {
    Table,
    CSV,
}

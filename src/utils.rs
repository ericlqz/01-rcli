use anyhow::Result;
use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
};

pub fn read_data(input: &str) -> Result<Box<dyn Read + 'static>> {
    let reader: Box<dyn Read> = if input == "-" {
        Box::new(std::io::stdin())
    } else if Path::new(input).exists() {
        Box::new(File::open(input)?)
    } else {
        let bytes = input.as_bytes().to_vec();
        let cursor = Cursor::new(bytes);
        Box::new(cursor)
    };

    Ok(reader)
}

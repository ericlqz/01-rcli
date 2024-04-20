use anyhow::Result;
use std::{fs::File, io::Read};

pub fn read_data(input: &str) -> Result<Box<dyn Read>> {
    // TODO: 如何使用box消除两种不同返回类型
    let reader: Box<dyn Read> = if input == "-" {
        Box::new(std::io::stdin())
    } else {
        Box::new(File::open(input)?)
    };

    Ok(reader)
}

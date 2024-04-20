use anyhow::Result;
use csv::Reader;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

use crate::OutputFormat;

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Position")]
    position: String,
    #[serde(rename = "DOB")]
    dob: String,
    #[serde(rename = "Nationality")]
    nationality: String,
    #[serde(rename = "Kit Number")]
    kit: u8,
}

pub fn process_csv(input: &str, output: String, output_format: OutputFormat) -> Result<()> {
    let mut reader = Reader::from_path(input).unwrap();
    // let players = reader
    //     .deserialize()
    //     .map(|record| record.unwrap())
    //     .collect::<Vec<Player>>();

    let mut ret = Vec::with_capacity(128);
    let headers = reader.headers()?.clone();

    for des_read in reader.records() {
        let record = des_read?;
        let json_record = headers.iter().zip(record.iter()).collect::<Value>();
        ret.push(json_record);
    }

    let content = match output_format {
        OutputFormat::Json => serde_json::to_string_pretty(&ret)?,
        OutputFormat::Yaml => serde_yaml::to_string(&ret)?,
    };
    fs::write(output, content)?;

    Ok(())
}

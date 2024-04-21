mod base64;
mod csv;
mod genpass;
mod http;
mod text;

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

pub use self::{
    base64::{Base64Format, Base64SubCommand},
    csv::{CsvOpts, OutputFormat},
    genpass::GenPassOpts,
    http::HttpSubCommand,
    text::{TextSignFormat, TextSubCommand},
};

// pub use {self::base64::Base64Format, self::csv::OutputFormat};

// rcli csv -i input.csv -o output.json --header -d ','

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Subcommand)]
pub enum SubCommand {
    #[command(name = "csv", about = "Convert csv to another format")]
    Csv(CsvOpts),

    #[command(name = "genpass", about = "Generate password")]
    GenPass(GenPassOpts),

    #[command(subcommand)]
    Base64(Base64SubCommand),

    #[command(subcommand)]
    Text(TextSubCommand),

    #[command(subcommand)]
    Http(HttpSubCommand),
}

pub fn verify_file(file_name: &str) -> Result<String, String> {
    if file_name == "-" || Path::new(file_name).exists() {
        Ok(file_name.into())
    } else {
        Err("File does not exists.".into())
    }
}

pub fn verify_path(path: &str) -> Result<PathBuf, String> {
    let p = Path::new(path);
    if p.exists() && p.is_dir() {
        Ok(path.into())
    } else {
        Err("File does not exists.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_input_file() {
        assert_eq!(verify_file("-"), Ok("-".into()));
        assert_eq!(verify_file("Cargo.toml"), Ok("Cargo.toml".into()));
        assert_eq!(
            verify_file("not-exists"),
            Err("File does not exists.".into())
        );
    }
}

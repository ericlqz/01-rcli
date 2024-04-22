use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use clap::Parser;
use enum_dispatch::enum_dispatch;
use std::{fmt, fs, path::PathBuf, str::FromStr};

use crate::{
    process_key_generate, process_text_decrypt, process_text_encrypt, process_text_sign,
    process_text_verify, CmdExecutor,
};

use super::{verify_file, verify_path};

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExecutor)]
pub enum TextSubCommand {
    #[command(about = "Sign a message with private/shared key")]
    Sign(TextSignOpts),

    #[command(about = "Verify a signed message")]
    Verify(TextVerifyOpts),

    #[command(about = "Generate a new key")]
    Generate(GenerateOpts),

    #[command(about = "Encrypt message")]
    Encrypt(EncryptOpts),

    #[command(about = "Decrypt message")]
    Decrypt(DecryptOpts),
}

#[derive(Debug, Parser)]
pub struct TextSignOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(short, long, value_parser = verify_file)]
    pub key: String,

    #[arg(long, default_value = "blake3", value_parser = verify_format)]
    pub format: TextSignFormat,
}

#[derive(Debug, Parser)]
pub struct TextVerifyOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(short, long, value_parser = verify_file)]
    pub key: String,

    #[arg(long, default_value = "blake3", value_parser = verify_format)]
    pub format: TextSignFormat,

    #[arg(short, long)]
    pub sig: String,
}

#[derive(Debug, Parser)]
pub struct GenerateOpts {
    #[arg(long, default_value = "blake3", value_parser = verify_format)]
    pub format: TextSignFormat,

    #[arg(short, long, value_parser = verify_path)]
    pub output: PathBuf,
}

#[derive(Debug, Parser)]
pub struct EncryptOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(long, default_value = "chacha20poly1305", value_parser = verify_encrypt_format)]
    pub format: TextEncryptFormat,

    #[arg(short, long, value_parser = verify_file)]
    pub key: String,
}

#[derive(Debug, Parser)]
pub struct DecryptOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(long, default_value = "chacha20poly1305", value_parser = verify_encrypt_format)]
    pub format: TextEncryptFormat,

    #[arg(short, long, value_parser = verify_file)]
    pub key: String,
}

#[derive(Debug, Clone, Copy, Parser)]
pub enum TextSignFormat {
    Blake3,  // 哈希算法
    Ed25519, // 数字签名算法
}

pub fn verify_format(ft: &str) -> Result<TextSignFormat, anyhow::Error> {
    ft.parse()
}

impl From<TextSignFormat> for &'static str {
    fn from(format: TextSignFormat) -> Self {
        match format {
            TextSignFormat::Blake3 => "blake3",
            TextSignFormat::Ed25519 => "ed25519",
        }
    }
}

impl FromStr for TextSignFormat {
    type Err = anyhow::Error;

    fn from_str(format: &str) -> Result<Self, Self::Err> {
        match format.to_lowercase().as_str() {
            "blake3" => Ok(TextSignFormat::Blake3),
            "ed25519" => Ok(TextSignFormat::Ed25519),
            v => anyhow::bail!("Unsupported format {:?}", v),
        }
    }
}

impl fmt::Display for TextSignFormat {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", Into::<&str>::into(*self))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TextEncryptFormat {
    ChaCha20Poly1305,
}

pub fn verify_encrypt_format(ft: &str) -> Result<TextEncryptFormat, anyhow::Error> {
    ft.parse()
}

impl From<TextEncryptFormat> for &'static str {
    fn from(format: TextEncryptFormat) -> Self {
        match format {
            TextEncryptFormat::ChaCha20Poly1305 => "chacha20poly1305",
        }
    }
}

impl FromStr for TextEncryptFormat {
    type Err = anyhow::Error;

    fn from_str(format: &str) -> Result<Self, Self::Err> {
        match format.to_lowercase().as_str() {
            "chacha20poly1305" => Ok(TextEncryptFormat::ChaCha20Poly1305),
            v => anyhow::bail!("Unsupported format {:?}", v),
        }
    }
}

impl fmt::Display for TextEncryptFormat {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", Into::<&str>::into(*self))
    }
}

impl CmdExecutor for TextSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let signed = process_text_sign(&self.input, &self.key, self.format)?;
        let signed = URL_SAFE_NO_PAD.encode(signed);
        println!("{}", signed);
        Ok(())
    }
}

impl CmdExecutor for TextVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let verified = process_text_verify(&self.input, &self.key, self.format, &self.sig)?;
        println!("{}", verified);
        Ok(())
    }
}

impl CmdExecutor for GenerateOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let keys = process_key_generate(self.format)?;
        match self.format {
            TextSignFormat::Blake3 => {
                let name = self.output.join("blake3.txt");
                fs::write(name, &keys[0])?;
            }
            TextSignFormat::Ed25519 => {
                let name = self.output;
                fs::write(name.join("ed25519.sk"), &keys[0])?;
                fs::write(name.join("ed25519.pk"), &keys[1])?;
            }
        }
        Ok(())
    }
}

impl CmdExecutor for EncryptOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let encrypted = process_text_encrypt(&self.input, &self.key, self.format)?;
        let encrypted = String::from_utf8(encrypted)?;
        println!("\nEncrypted msg: {}", encrypted);
        Ok(())
    }
}

impl CmdExecutor for DecryptOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let decrypted = process_text_decrypt(&self.input, &self.key, self.format)?;
        let decrypted = String::from_utf8(decrypted)?;
        println!("\nDecrypted msg: {}", decrypted);
        Ok(())
    }
}

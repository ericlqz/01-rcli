use crate::CmdExecutor;
use chrono::{Duration, Local};
use clap::Parser;
use enum_dispatch::enum_dispatch;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExecutor)]
pub enum JwtSubCommand {
    #[command(name = "sign", about = "Sign jwt")]
    Sign(JwtSignOpts),

    #[command(name = "verify", about = "Verify jwt")]
    Verify(JwtVerifyOpts),
}

#[derive(Debug, Parser)]
pub struct JwtSignOpts {
    #[arg(short, long, default_value = "jpN81Vuv$hUu_4A3JwUeoE7^@a6Mnd&j")]
    pub key: String,

    #[arg(short, long)]
    pub sub: String,

    #[arg(short, long)]
    pub aud: String,

    #[arg(short, long, value_parser = parse_expire_time, default_value = "5m")]
    pub exp: usize,
}

#[derive(Debug, Parser)]
pub struct JwtVerifyOpts {
    #[arg(short, long, default_value = "jpN81Vuv$hUu_4A3JwUeoE7^@a6Mnd&j")]
    pub key: String,

    #[arg(short, long)]
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    aud: String,
    exp: usize,
}

fn parse_expire_time(duration_str: &str) -> anyhow::Result<usize, anyhow::Error> {
    let re = Regex::new(r"^(\d+)([smhd])$").unwrap();

    if let Some(captures) = re.captures(duration_str) {
        let value: i64 = captures[1].parse().ok().unwrap();
        let duration = match &captures[2] {
            "s" => Duration::seconds(value),
            "m" => Duration::minutes(value),
            "h" => Duration::hours(value),
            "d" => Duration::days(value),
            _ => Duration::minutes(5),
        };
        let expire_time = Local::now() + duration;
        debug!(
            "Current time: {}, duration: {}, expire time: {}",
            Local::now(),
            duration_str,
            expire_time
        );
        Ok(expire_time.timestamp() as usize)
    } else {
        anyhow::bail!("Input duration invalid. valid example: 10d/2h/20m/60s")
    }
}

impl CmdExecutor for JwtSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let claims = Claims {
            sub: self.sub.clone(),
            aud: self.aud.clone(),
            exp: self.exp,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.key.as_ref()),
        )?;
        println!("jwt sign token: {:?}", token);
        Ok(())
    }
}

impl CmdExecutor for JwtVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_aud = false;
        let token_message = decode::<Claims>(
            &self.token,
            &DecodingKey::from_secret(self.key.as_ref()),
            &validation,
        )?;
        println!("jwt verify token: {:?}", token_message.claims);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expire_time() {
        parse_expire_time("30s").unwrap();
    }
}

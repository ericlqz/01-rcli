use std::{fs, io::Read, path::Path};

use crate::{process_genpass, read_data, TextEncryptFormat, TextSignFormat};
use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

pub fn process_text_sign(input: &str, key: &str, format: TextSignFormat) -> Result<Vec<u8>> {
    let mut reader = read_data(input)?;

    let signed = match format {
        TextSignFormat::Blake3 => {
            let signer = Blake3::load(key)?;
            signer.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => {
            let signer = Ed25519Signer::load(key)?;
            signer.sign(&mut reader)?
        }
    };

    Ok(signed)
}

pub fn process_text_verify(
    input: &str,
    key: &str,
    format: TextSignFormat,
    sig: &str,
) -> Result<bool> {
    let mut reader = read_data(input)?;

    let sig = URL_SAFE_NO_PAD.decode(sig)?;

    let verified = match format {
        TextSignFormat::Blake3 => {
            let verifier = Blake3::load(key)?;
            verifier.verify(&mut reader, &sig)?
        }
        TextSignFormat::Ed25519 => {
            let verifier = Ed25519Verifier::load(key)?;
            verifier.verify(&mut reader, &sig)?
        }
    };

    Ok(verified)
}

pub fn process_key_generate(format: TextSignFormat) -> Result<Vec<Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
    }
}

pub fn process_text_encrypt(input: &str, key: &str, format: TextEncryptFormat) -> Result<Vec<u8>> {
    let mut reader = read_data(input)?;

    let encrypted = match format {
        TextEncryptFormat::ChaCha20Poly1305 => {
            let signer = ChaCha20::load(key)?;
            signer.encrypt(&mut reader)?
        }
    };

    Ok(encrypted)
}

pub fn process_text_decrypt(input: &str, key: &str, format: TextEncryptFormat) -> Result<Vec<u8>> {
    let mut reader = read_data(input)?;

    let decrypted = match format {
        TextEncryptFormat::ChaCha20Poly1305 => {
            let signer = ChaCha20::load(key)?;
            signer.decrypt(&mut reader)?
        }
    };

    Ok(decrypted)
}

pub trait TextSign {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerify {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool>;
}

pub trait TextEncryptDecrypt {
    fn encrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
    fn decrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait KeyLoader {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
}

pub trait KeyGenerator {
    fn generate() -> Result<Vec<Vec<u8>>>;
}

pub struct Blake3 {
    key: [u8; 32],
}

pub struct Ed25519Signer {
    key: SigningKey,
}

pub struct Ed25519Verifier {
    key: VerifyingKey,
}

pub struct ChaCha20 {
    key: Key,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChaCha20EncryptedData {
    encrypt_data: Vec<u8>,
    // nonce: Nonce,
    nonce: Vec<u8>,
}

impl ChaCha20EncryptedData {
    fn try_new_from_base64(base_str: String) -> Result<Self> {
        let base64_decode_str = URL_SAFE_NO_PAD.decode(base_str)?;
        let encrypted_data: ChaCha20EncryptedData = serde_json::from_slice(&base64_decode_str)?;
        Ok(encrypted_data)
    }

    fn to_base64(&self) -> Result<String> {
        let serialized = serde_json::to_string(self)?;
        let base64_str = URL_SAFE_NO_PAD.encode(serialized);

        Ok(base64_str)
    }
}

impl TextSign for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        // TODO: improve perf by reading in chunks
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(blake3::keyed_hash(&self.key, &buf).as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let hash = blake3::keyed_hash(&self.key, &buf);
        let hash = hash.as_bytes();

        Ok(hash == sig)
    }
}

impl KeyLoader for Blake3 {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_genpass(32, true, true, true, true)?;
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

impl TextSign for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let sig = self.key.sign(&buf);
        Ok(sig.to_bytes().to_vec())
    }
}

impl KeyLoader for Ed25519Signer {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut csprng = OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let pk = sk.verifying_key().as_bytes().to_vec();
        let sk = sk.as_bytes().to_vec();
        Ok(vec![sk, pk])
    }
}

impl TextVerify for Ed25519Verifier {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = Signature::from_bytes(sig.try_into()?);

        let ret = self.key.verify_strict(&buf, &sig).is_ok();
        Ok(ret)
    }
}

impl KeyLoader for Ed25519Verifier {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl TextEncryptDecrypt for ChaCha20 {
    fn encrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let cipher: ChaCha20Poly1305 = ChaCha20Poly1305::new(&self.key);
        let nonce: Nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let encrypted = cipher.encrypt(&nonce, &buf as &[u8]).unwrap();
        let data = ChaCha20EncryptedData {
            encrypt_data: encrypted,
            nonce: nonce.to_vec(),
        }
        .to_base64()?;

        Ok(data.into_bytes().to_vec())
    }

    fn decrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let base = String::from_utf8(buf)?;
        let data = ChaCha20EncryptedData::try_new_from_base64(base)?;
        let cipher: ChaCha20Poly1305 = ChaCha20Poly1305::new(&self.key);
        let nonce = *Nonce::from_slice(&data.nonce);
        let plaintext = cipher.decrypt(&nonce, &data.encrypt_data as &[u8]).unwrap();

        Ok(plaintext)
    }
}

impl KeyLoader for ChaCha20 {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl Blake3 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = &key[..32];
        let key = key.try_into()?;
        let signer = Blake3::new(key);
        Ok(signer)
    }
}

impl Ed25519Signer {
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = SigningKey::from_bytes(key.try_into()?);
        let signer = Ed25519Signer::new(key);
        Ok(signer)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl Ed25519Verifier {
    pub fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = VerifyingKey::from_bytes(key.try_into()?)?;
        let verifier = Ed25519Verifier::new(key);
        Ok(verifier)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl ChaCha20 {
    pub fn new(key: Key) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = &key[..32];
        let key: [u8; 32] = key.try_into()?;
        let key = Key::from(key);
        let signer = ChaCha20::new(key);
        Ok(signer)
    }
}

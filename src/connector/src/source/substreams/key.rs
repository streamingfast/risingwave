use anyhow::Result;
use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use sodiumoxide::crypto::secretbox;
use std::error::Error;

// Yes, this is hardcoded, no big deal.
// We do not really care about Confidentiality - this is merely for obfuscation
// Was generated with `openssl rand -hex 32`
const OPAQUE_CURSOR_ENCRYPTION_KEY: [u8; 32] = [
    0x7b, 0xfc, 0xac, 0xee, 0x25, 0x74, 0x09, 0x09, 0x9e, 0xdd, 0x2b, 0xb6, 0xa4, 0x42, 0x63, 0x8b,
    0x55, 0x9a, 0x80, 0xbf, 0xbf, 0xc0, 0xb9, 0xac, 0xde, 0xa0, 0xd8, 0x34, 0x4b, 0x10, 0xeb, 0x00,
];

// **WARNING** - DO NOT copy this unless you know what you are doing
// We are using a fixed nonce by design here.
// Using a fixed nonce is FATAL cryptography security flaw in normal cases
// But in this case we mostly care of obscuring / making opaque the key
const FIXED_NONCE: [u8; 24] = [
    0x26, 0x15, 0x54, 0xc4, 0x5a, 0xb9, 0xb7, 0x52, 0xab, 0xad, 0x4f, 0x19, 0xc2, 0x42, 0x60, 0x57,
    0x02, 0xd5, 0x5a, 0x0d, 0x91, 0x61, 0x6a, 0x1b,
];

pub fn encode(internal_key: &[u8]) -> String {
    let nonce = secretbox::Nonce::from_slice(&FIXED_NONCE).unwrap();
    let key = secretbox::Key::from_slice(&OPAQUE_CURSOR_ENCRYPTION_KEY).unwrap();
    let ciphertext = secretbox::seal(internal_key, &nonce, &key);
    URL_SAFE.encode(ciphertext)
}

pub fn encode_string(internal_key: &str) -> String {
    encode(internal_key.as_bytes())
}

pub fn decode(opaque_key: &str) -> Result<Vec<u8>> {
    let ciphertext = URL_SAFE.decode(opaque_key)?;
    let nonce = secretbox::Nonce::from_slice(&FIXED_NONCE).unwrap();
    let key = secretbox::Key::from_slice(&OPAQUE_CURSOR_ENCRYPTION_KEY).unwrap();

    secretbox::open(&ciphertext, &nonce, &key)
        .map_err(|_| anyhow::Error::msg("decryption failed"))
}

pub fn decode_to_string(opaque_key: &str) -> Result<String> {
    let bytes = decode(opaque_key)?;
    String::from_utf8(bytes).map_err(Into::into)
}

pub fn to_opaque(internal_key: &str) -> Result<String> {
    Ok(encode_string(internal_key))
}

pub fn from_opaque(opaque_key: &str) -> Result<String> {
    decode_to_string(opaque_key)
}
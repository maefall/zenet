use crate::ZauthError;
use hmac::{digest::FixedOutput, Hmac, Mac};
use sha2::Sha256;
use zwire::codec::bytes::{string::ByteStr, Bytes};

type HmacSha256 = Hmac<Sha256>;

pub fn auth_mac(
    key: &[u8],
    client_identifier: &ByteStr,
    timestamp: u64,
    nonce: u128,
) -> Result<Bytes, ZauthError> {
    let mut mac = HmacSha256::new_from_slice(key)?;
    let client_id_bytes = client_identifier.as_bytes();

    mac.update(&(client_id_bytes.len() as u16).to_be_bytes());
    mac.update(client_id_bytes);

    mac.update(&timestamp.to_be_bytes());
    mac.update(&nonce.to_be_bytes());

    let tag_array = mac.finalize_fixed();

    Ok(Bytes::copy_from_slice(&tag_array))
}

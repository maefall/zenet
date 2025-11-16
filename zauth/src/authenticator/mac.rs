use crate::ZauthError;
use tokio_util::bytes::Bytes;

use hmac::{digest::FixedOutput, Hmac, Mac};
use sha2::Sha256;

pub fn auth_mac(
    key: &[u8],
    client_identifier: &str,
    timestamp: u64,
    nonce: &Bytes,
) -> Result<Bytes, ZauthError> {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(key)?;
    let client_id_bytes = client_identifier.as_bytes();

    mac.update(&(client_id_bytes.len() as u16).to_be_bytes());
    mac.update(client_id_bytes);

    mac.update(&timestamp.to_be_bytes());
    mac.update(nonce);

    let tag_array = mac.finalize_fixed();

    Ok(Bytes::copy_from_slice(&tag_array))
}

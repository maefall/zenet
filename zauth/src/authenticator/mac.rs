use crate::{ZauthError, MAC_LENGTH, NONCE_LENGTH};
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn auth_mac(
    key: &[u8],
    client_identifier: &str,
    timestamp: u64,
    nonce16: &[u8; NONCE_LENGTH],
) -> Result<[u8; MAC_LENGTH], ZauthError> {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(key)?;
    let client_id_bytes = client_identifier.as_bytes();

    mac.update(&(client_id_bytes.len() as u16).to_be_bytes());
    mac.update(client_id_bytes);

    mac.update(&timestamp.to_be_bytes());
    mac.update(nonce16);

    let out = mac.finalize().into_bytes();

    let mut tag = [0u8; MAC_LENGTH];

    tag.copy_from_slice(&out);

    Ok(tag)
}

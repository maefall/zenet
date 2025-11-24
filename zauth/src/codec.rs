use crate::AuthPayload;
use tokio_util::{
    bytes::{Buf, BytesMut},
    codec::{Decoder, Encoder},
};
use zwire::{
    codec::bytestring::{ByteStringFieldExt, ByteStringFieldPolicy},
    codec::{define_fields, BytesMutPutExt, BytesPeekExt, CheckedAddWire, WiredInt},
    errors::WireError,
    DecodeFromFrame, EncodeIntoFrame,
};

impl EncodeIntoFrame for AuthPayloadCodec {
    type EncodeItem = AuthPayload;
}

impl DecodeFromFrame for AuthPayloadCodec {}

#[derive(Clone, Copy)]
pub struct AuthPayloadCodec {
    max_length: usize,
}

impl Default for AuthPayloadCodec {
    fn default() -> Self {
        Self {
            max_length: FIXED_PART_LENGTH + MAX_CLIENT_IDENTIFIER_LENGTH + MAC_LENGTH,
        }
    }
}

const MAX_CLIENT_IDENTIFIER_LENGTH: usize = 255;
const MAC_LENGTH: usize = 32; // what could we do about this?

// [u64 timestamp] | [u128 nonce] | [mac] | [u8 length][client_id...]|
define_fields! {
    (Timestamp, u64, 0, fixed),
    (Nonce, u128, 8, fixed),
    // (Mac, usize, 24),
    (ClientIdentifier, u8, 56, length_prefix),
}

impl Encoder<AuthPayload> for AuthPayloadCodec {
    type Error = WireError;

    fn encode(
        &mut self,
        auth_payload: AuthPayload,
        destination: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let client_id_bytes = auth_payload.client_identifier.as_bytes();
        let client_id_length = client_id_bytes.len();

        if client_id_length > MAX_CLIENT_IDENTIFIER_LENGTH {
            return Err(WireError::Oversized(
                "client_identifier_length",
                client_id_length,
                MAX_CLIENT_IDENTIFIER_LENGTH,
            ));
        }

        let total_length = (FIXED_PART_LENGTH + MAC_LENGTH).checked_add_wire(
            "FIXED_PART_LENGTH + MAC_LENGTH",
            client_id_length,
            "client_id_length",
        )?;

        if total_length > self.max_length {
            return Err(WireError::Oversized(
                "total_length",
                total_length,
                self.max_length,
            ));
        }

        destination.put_single::<TimestampWired>(auth_payload.timestamp);
        destination.put_single::<NonceWired>(auth_payload.nonce);
        destination.append_bytes(&auth_payload.mac, MAC_LENGTH, "mac", None)?;

        destination.put_length_prefixed::<ClientIdentifierWired>(
            client_id_bytes,
            "client_identifier",
            None,
        )?;

        Ok(())
    }
}

impl Decoder for AuthPayloadCodec {
    type Item = AuthPayload;
    type Error = WireError;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let Some(client_id_length) = source
            .peek_at::<ClientIdentifierWired>(
                CLIENTIDENTIFIER_FIELD_OFFSET,
                "client_identifier_length",
            )?
            .get()
        else {
            return Ok(None);
        };

        let total_length = (FIXED_PART_LENGTH + MAC_LENGTH).checked_add_wire(
            "FIXED_PART_LENGTH + MAC_LENGTH",
            client_id_length,
            "client_id_length",
        )?;

        if total_length > self.max_length {
            return Err(WireError::Oversized(
                "total_length",
                total_length,
                self.max_length,
            ));
        }

        if source.len() < total_length {
            return Ok(None);
        }

        let timestamp = source.get_u64();
        let nonce = source.get_u128();

        let tail_len = MAC_LENGTH + ClientIdentifierWired::SIZE + client_id_length;

        let mut tail = source.split_to(tail_len);

        let mac = tail.split_to(MAC_LENGTH).freeze();

        tail.advance(ClientIdentifierWired::SIZE);

        let client_identifier_bytes = tail.split_to(client_id_length).freeze();
        let client_identifier = client_identifier_bytes.to_bytestr_field(
            "client_identifier",
            Some(MAX_CLIENT_IDENTIFIER_LENGTH),
            ByteStringFieldPolicy::AsciiHyphen,
        )?;

        Ok(Some(AuthPayload {
            client_identifier,
            timestamp,
            nonce,
            mac,
        }))
    }
}

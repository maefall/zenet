use crate::AuthPayload;
use tokio_util::{
    bytes::{Buf, BytesMut},
    codec::{Decoder, Encoder},
};
use zwire::{
    codec::bytestring::{ByteStringFieldExt, ByteStringFieldPolicy},
    codec::{define_fields, BytesMutPutExt, BytesPeekExt, LengthPrefix},
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
            max_length: HEADER_LENGTH + MAX_CLIENT_IDENTIFIER_LENGTH +  NONCE_LENGTH + MAC_LENGTH,
        }
    }
}

const MAX_CLIENT_IDENTIFIER_LENGTH: usize = 255;
const NONCE_LENGTH: usize = 16;
const MAC_LENGTH: usize = 32;

define_fields! {
    (Timestamp, u64, 0),
    (ClientIdentifier, u8, 8),
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

        let total_length = (HEADER_LENGTH + NONCE_LENGTH + MAC_LENGTH)
            .checked_add(client_id_length)
            .ok_or(WireError::Oversized(
                "total_length",
                client_id_length,
                self.max_length,
            ))?;

        if total_length > self.max_length {
            return Err(WireError::Oversized(
                "total_length",
                total_length,
                self.max_length,
            ));
        }

        destination.put_single::<TimestampLengthPrefix>(auth_payload.timestamp);
        destination.put_length_prefixed::<ClientIdentifierLengthPrefix>(
            client_id_bytes,
            "client_identifier",
            None,
        )?;

        destination.append_bytes(&auth_payload.nonce, NONCE_LENGTH, "nonce", None)?;
        destination.append_bytes(&auth_payload.mac, MAC_LENGTH, "mac", None)?;

        Ok(())
    }
}

impl Decoder for AuthPayloadCodec {
    type Item = AuthPayload;
    type Error = WireError;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let source_length = source.len();

        if source_length < ClientIdentifierLengthPrefix::WIDTH {
            return Ok(None);
        }

        let Some(client_id_length) = source.peek_at::<u8>(CLIENTIDENTIFIER_FIELD_OFFSET).get()
        else {
            return Ok(None);
        };

        let total_length = HEADER_LENGTH + client_id_length + NONCE_LENGTH + MAC_LENGTH;

        if total_length > self.max_length {
            return Err(WireError::Oversized(
                "total_length",
                total_length,
                self.max_length,
            ));
        }

        if source_length < total_length {
            return Ok(None);
        }

        let mut frame = source.split_to(total_length);
        frame.advance(ClientIdentifierLengthPrefix::WIDTH);

        let timestamp = frame.get_u64();
        let client_identifier_bytes = frame.split_to(client_id_length).freeze();
        let nonce = frame.split_to(NONCE_LENGTH).freeze();
        let mac = frame.split_to(MAC_LENGTH).freeze();

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

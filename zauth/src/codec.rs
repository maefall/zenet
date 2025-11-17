use crate::{
    AuthPayload, CLIENT_IDENTIFIER_LENGTH_FIELD_OFFSET, CLIENT_IDENTIFIER_LENGTH_HEADER_LENGTH,
    FIXED_PART_LENGTH, MAC_LENGTH, MAX_CLIENT_IDENTIFIER_LENGTH, NONCE_LENGTH, TIMESTAMP_LENGTH,
};
use tokio_util::{
    bytes::{Buf, BufMut, BytesMut},
    codec::{Decoder, Encoder},
};
use zwire::{
    codec::bytestring::{ByteStringFieldExt, ByteStringFieldPolicy},
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
            max_length: usize::MAX,
        }
    }
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

        let total_length =
            FIXED_PART_LENGTH
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

        destination.put_u8(client_id_length as u8);
        destination.extend_from_slice(client_id_bytes);
        destination.put_u64(auth_payload.timestamp);
        destination.extend_from_slice(&auth_payload.nonce);
        destination.extend_from_slice(&auth_payload.mac);

        Ok(())
    }
}

impl Decoder for AuthPayloadCodec {
    type Item = AuthPayload;
    type Error = WireError;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let source_length = source.len();

        if source_length < CLIENT_IDENTIFIER_LENGTH_HEADER_LENGTH {
            return Ok(None);
        }

        let client_id_length =
            u8::from_be_bytes([source[CLIENT_IDENTIFIER_LENGTH_FIELD_OFFSET]]) as usize;

        let total_length = CLIENT_IDENTIFIER_LENGTH_HEADER_LENGTH
            + client_id_length
            + TIMESTAMP_LENGTH
            + NONCE_LENGTH
            + MAC_LENGTH;

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
        frame.advance(CLIENT_IDENTIFIER_LENGTH_HEADER_LENGTH);

        let client_identifier_bytes = frame.split_to(client_id_length).freeze();
        let timestamp = frame.get_u64();
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

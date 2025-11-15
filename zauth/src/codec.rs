use crate::{
    AuthPayload, CLIENT_ID_LENGTH_FIELD_OFFSET, CLIENT_ID_LENGTH_HEADER_LENGTH, FIXED_PART_LENGTH,
    MAC_LENGTH, MAX_CLIENT_IDENTIFIER_LENGTH, NONCE_LENGTH, TIMESTAMP_LENGTH,
};
use tokio_util::{
    bytes::{Buf, BufMut, BytesMut},
    codec::{Decoder, Encoder},
};
use zwire::{errors::WireError, DecodeFromFrame, EncodeIntoFrame, Frame, MessageType};

impl EncodeIntoFrame for AuthPayloadCodec {
    type Item = AuthPayload;

    fn encode_into_frame(
        &mut self,
        payload: Self::Item,
        message_type: MessageType,
        codec_buffer: &mut BytesMut,
    ) -> Result<Frame, WireError> {
        let start_offset = codec_buffer.len();

        self.encode(payload, codec_buffer)?;

        let auth_payload_bytes = codec_buffer.split_off(start_offset);

        Ok(Frame {
            message_type,
            payload: auth_payload_bytes.freeze(),
        })
    }
}

impl DecodeFromFrame for AuthPayloadCodec {
    type Item = AuthPayload;

    fn decode_from_frame(
        &mut self,
        frame: Frame,
        codec_buffer: &mut BytesMut,
    ) -> Result<Option<(Self::Item, MessageType)>, WireError> {
        codec_buffer.extend_from_slice(&frame.payload);

        if let Some(auth_payload) = self.decode(codec_buffer)? {
            Ok(Some((auth_payload, frame.message_type)))
        } else {
            Ok(None)
        }
    }
}

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

        destination.put_u16(client_id_length as u16);
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

        if source_length < CLIENT_ID_LENGTH_HEADER_LENGTH {
            return Ok(None);
        }

        let client_id_length = u16::from_be_bytes([
            source[CLIENT_ID_LENGTH_FIELD_OFFSET],
            source[CLIENT_ID_LENGTH_FIELD_OFFSET + 1],
        ]) as usize;

        let total_length = CLIENT_ID_LENGTH_HEADER_LENGTH
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

        frame.advance(CLIENT_ID_LENGTH_HEADER_LENGTH);

        let id_bytes = frame.split_to(client_id_length);
        let timestamp = u64::from_be_bytes(
            frame
                .split_to(TIMESTAMP_LENGTH)
                .as_ref()
                .try_into()
                .unwrap(),
        );
        let nonce: [u8; NONCE_LENGTH] = frame.split_to(NONCE_LENGTH).as_ref().try_into().unwrap();
        let mac: [u8; MAC_LENGTH] = frame.split_to(MAC_LENGTH).as_ref().try_into().unwrap();

        let client_identifier = std::str::from_utf8(&id_bytes)
            .map_err(|_| WireError::MalformedString("client_identifier"))?
            .to_string();

        Ok(Some(AuthPayload {
            client_identifier,
            timestamp,
            nonce,
            mac,
        }))
    }
}

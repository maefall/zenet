use crate::AuthPayload;
use zwire::{
    codec::{
        bytes::{
            string::{ByteStringFieldExt, ByteStringFieldPolicy},
            BytesMut, BytesMutPutExt, BytesMutTakeExt, BytesPeekExt,
        },
        wired::define_fields,
        Decoder, Encoder,
    },
    errors::WireError,
    helpers::CheckedAddWire,
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
            max_length: fields::MAX_LENGTH,
        }
    }
}

// [u64 timestamp] | [u128 nonce] | [mac] | [u8 length][client_id...]
define_fields! {
    (Timestamp, u64, fixed),
    (Nonce, u128, fixed),
    (Mac, 32, fixed),
    (ClientIdentifier, u8, length_prefix, 255),
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
        let client_id_max_length = fields::clientidentifier::MAX_LENGTH;

        if client_id_length > client_id_max_length {
            return Err(WireError::Oversized(
                "client_identifier_length",
                client_id_length,
                client_id_max_length,
            ));
        }

        let total_length = fields::FIXED_PART_LENGTH.checked_add_wire(
            "FIXED_PART_LENGTH",
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

        destination.put_single::<fields::timestamp::Wired>(auth_payload.timestamp);
        destination.put_single::<fields::nonce::Wired>(auth_payload.nonce);
        destination.put_fixed_bytes::<fields::mac::Wired>(&auth_payload.mac)?;
        destination.put_length_prefixed::<fields::clientidentifier::Wired>(client_id_bytes)?;

        Ok(())
    }
}

impl Decoder for AuthPayloadCodec {
    type Item = AuthPayload;
    type Error = WireError;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let Some(client_id_length) = source
            .peek_at::<fields::clientidentifier::Wired>(
                fields::clientidentifier::OFFSET,
                "client_identifier_length",
            )?
            .get()
        else {
            return Ok(None);
        };

        let total_length = fields::FIXED_PART_LENGTH.checked_add_wire(
            "FIXED_PART_LENGTH",
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

        let timestamp = source.take_single_unchecked::<fields::timestamp::Wired>();
        let nonce = source.take_single_unchecked::<fields::nonce::Wired>();
        let mac = source.take_fixed_bytes_unchecked::<fields::mac::Wired>();
        let client_identifier_bytes =
            source.take_length_prefixed_unchecked::<fields::clientidentifier::Wired>()?;

        let client_identifier = client_identifier_bytes.to_bytestr_field(
            "client_identifier",
            None,
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

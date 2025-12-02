use crate::AudioPayload;
use zwire::{
    codec::{bytes::BytesMut, wired::define_fields, Decoder, Encoder},
    errors::WireError,
    DecodeFromFrame, EncodeIntoFrame,
};

impl EncodeIntoFrame for AudioPayloadCodec {
    type EncodeItem = AudioPayload;
}

impl DecodeFromFrame for AudioPayloadCodec {}

#[derive(Clone, Copy)]
pub struct AudioPayloadCodec {
    max_length: usize,
}

impl Default for AudioPayloadCodec {
    fn default() -> Self {
        Self {
            max_length: fields::MAX_LENGTH,
        }
    }
}

// [u64 timestamp] | [u128 nonce] | [mac] | [u8 length][client_id...]
define_fields! {
    (Timestamp, u64, fixed),
}

impl Encoder<AudioPayload> for AudioPayloadCodec {
    type Error = WireError;

    fn encode(
        &mut self,
        audio_payload: AudioPayload,
        destination: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Decoder for AudioPayloadCodec {
    type Item = AudioPayload;
    type Error = WireError;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}

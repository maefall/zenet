use crate::AudioPayload;
use zwire::{
    codec::{
        bytes::{BytesMut, BytesMutPutExt, BytesMutTakeExt},
        wired::define_fields,
        Decoder, Encoder,
    },
    errors::WireError,
    DecodeFromFrame, EncodeIntoFrame,
};

impl EncodeIntoFrame for AudioPayloadCodec {
    type EncodeItem = AudioPayload;
}

impl DecodeFromFrame for AudioPayloadCodec {}

#[derive(Clone, Copy, Default)]
pub struct AudioPayloadCodec {}

define_fields! {
    (Audio, u16, length_prefix, 65535),
}

impl Encoder<AudioPayload> for AudioPayloadCodec {
    type Error = WireError;

    fn encode(
        &mut self,
        audio_payload: AudioPayload,
        destination: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        destination.put_length_prefixed::<fields::audio::Wired>(&audio_payload)?;

        Ok(())
    }
}

impl Decoder for AudioPayloadCodec {
    type Item = AudioPayload;
    type Error = WireError;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let audio_payload = source.take_length_prefixed::<fields::audio::Wired>()?;

        Ok(audio_payload)
    }
}

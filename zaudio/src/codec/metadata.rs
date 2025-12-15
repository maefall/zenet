use crate::{AudioEncoding, AudioMetadata, Channels};
use zwire::{
    codec::{
        bytes::{BytesMut, BytesMutPutExt, BytesMutTakeExt},
        wired::define_fields,
        Decoder, Encoder,
    },
    errors::WireError,
    DecodeFromFrame, EncodeIntoFrame,
};

impl EncodeIntoFrame for AudioMetadataCodec {
    type EncodeItem = AudioMetadata;
}

impl DecodeFromFrame for AudioMetadataCodec {}

#[derive(Clone, Copy, Default)]
pub struct AudioMetadataCodec {}

define_fields! {
    (Encoding, u8, fixed),
    (Channels, u8, fixed),
    (sample_rate, u32, fixed),
}

impl Encoder<AudioMetadata> for AudioMetadataCodec {
    type Error = WireError;

    fn encode(
        &mut self,
        audio_metadata: AudioMetadata,
        destination: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        destination.put_single::<fields::encoding::Wired>(audio_metadata.encoding as u8);
        destination.put_single::<fields::channels::Wired>(audio_metadata.channels as u8);
        destination.put_single::<fields::sample_rate::Wired>(audio_metadata.sample_rate);

        Ok(())
    }
}

impl Decoder for AudioMetadataCodec {
    type Item = AudioMetadata;
    type Error = WireError;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let source_length = source.len();

        if source_length < fields::FIXED_PART_LENGTH {
            return Ok(None);
        }

        if source_length > fields::MAX_LENGTH {
            return Err(WireError::Oversized(
                "total_length",
                source_length,
                fields::MAX_LENGTH,
            ));
        }

        let encoding_code = source.take_single_unchecked::<fields::encoding::Wired>();
        let channels_code = source.take_single_unchecked::<fields::channels::Wired>();
        let sample_rate = source.take_single_unchecked::<fields::sample_rate::Wired>();

        Ok(Some(AudioMetadata {
            encoding: AudioEncoding::try_from(encoding_code)?,
            channels: Channels::try_from(channels_code)?,
            sample_rate,
        }))
    }
}

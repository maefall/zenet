mod codec;

pub use codec::{AudioMetadataCodec, AudioPayloadCodec};
use zwire::codec::{bytes::Bytes, wired::define_message};

pub mod __zwire_macros_support {
    pub use zwire::__zwire_macros_support::*;
}

type AudioPayload = Bytes;

define_message!(AudioEncoding, { PcmS16Le = 1 });

define_message!(
    Channels,
    {
        Mono = 1,
        Stereo = 2,
    }
);

#[derive(Debug, Clone)]
pub struct AudioMetadata {
    encoding: AudioEncoding,
    channels: Channels,
    sample_rate: u32,
}

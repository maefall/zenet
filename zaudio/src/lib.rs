mod codec;

pub use codec::{AudioMetadataCodec, AudioPayloadCodec};
use zwire::codec::{bytes::Bytes, wired::define_message};

pub mod __zwire_macros_support {
    pub use zwire::__zwire_macros_support::*;
}

type AudioPayload = Bytes;

// TODO: Make define_message more universal since we used it here to define types for encoding and
// channels
define_message!(AudioEncoding, { PcmS16Le = 1 });

define_message!(
    Channels,
    {
        Mono = 1,
        Stereo = 2,
    }
);

define_message!(
    ZaudioMessage,
    { 
        RequestTransmission = 1,
        ApproveTransmission = 2, // server opens uni 
    }
);

#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub encoding: AudioEncoding,
    pub channels: Channels,
    pub sample_rate: u32,
}

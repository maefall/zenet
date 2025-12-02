mod codec;

pub use codec::AudioPayloadCodec;

pub mod __zwire_macros_support {
    pub use zwire::__zwire_macros_support::*;
}

use zwire::codec::bytes::Bytes;

#[derive(Debug, Clone)]
pub enum AudioEncoding {
    PcmS16Le,
}

#[derive(Debug, Clone)]
pub enum Channels {
    Mono,
    Stereo,
}

#[derive(Debug, Clone)]
pub struct AudioMetaData {
    encoding: AudioEncoding,
    channels: Channels,
    sample_rate: u32,
}

#[derive(Debug, Clone)]
pub struct AudioPayload {
    audio: Bytes,
}

impl AudioPayload {
    pub fn new() -> Self {
        todo!()
    }
}

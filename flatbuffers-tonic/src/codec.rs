use std::marker::PhantomData;

use bytes::{Buf, BufMut};
use tonic::{
    Status,
    codec::{BufferSettings, Codec, Decoder, EncodeBuf, Encoder},
};

use crate::OwnedFBCodecable;

#[derive(Debug, Clone)]
pub struct FlatBuffersCodec<T, U> {
    _pd: PhantomData<(T, U)>,
}

impl<T, U> FlatBuffersCodec<T, U> {
    /// Configure a FlatBuffersCodec with encoder/decoder buffer settings. This is used to control
    /// how memory is allocated and grows per RPC.
    pub fn new() -> Self {
        Self { _pd: PhantomData }
    }
}

impl<T, U> Default for FlatBuffersCodec<T, U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, U> Codec for FlatBuffersCodec<T, U>
where
    T: OwnedFBCodecable + Send + 'static,
    U: OwnedFBCodecable + Send + 'static,
{
    type Encode = T;
    type Decode = U;

    type Encoder = FlatBuffersEncoder<T>;
    type Decoder = FlatBuffersDecoder<U>;

    fn encoder(&mut self) -> Self::Encoder {
        FlatBuffersEncoder {
            _pd: PhantomData,
            buffer_settings: BufferSettings::default(),
        }
    }

    fn decoder(&mut self) -> Self::Decoder {
        FlatBuffersDecoder {
            _pd: PhantomData,
            buffer_settings: BufferSettings::default(),
        }
    }
}

/// A [`Encoder`] that knows how to encode `T`.
#[derive(Debug, Clone, Default)]
pub struct FlatBuffersEncoder<T> {
    _pd: PhantomData<T>,
    buffer_settings: BufferSettings,
}

impl<T> Encoder for FlatBuffersEncoder<T>
where
    T: OwnedFBCodecable + Send + 'static,
{
    type Item = T;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        buf.put_slice(item.get_slice());
        Ok(())
    }

    fn buffer_settings(&self) -> BufferSettings {
        self.buffer_settings
    }
}

pub struct FlatBuffersDecoder<U> {
    _pd: PhantomData<U>,
    buffer_settings: BufferSettings,
}

impl<U: OwnedFBCodecable + Send + 'static> Decoder for FlatBuffersDecoder<U> {
    type Item = U;

    type Error = Status;

    fn decode(
        &mut self,
        src: &mut tonic::codec::DecodeBuf<'_>,
    ) -> Result<Option<Self::Item>, Self::Error> {
        // get the buf into a contiguous slice
        let len = src.remaining();
        let mut buf = vec![0u8; len];
        src.copy_to_slice(&mut buf);
        let owned_fb = U::new_boxed(buf.into_boxed_slice())
            .map_err(|e| Status::internal(format!("Failed to decode FlatBuffer: {}", e)))?;
        Ok(Some(owned_fb))
    }

    fn buffer_settings(&self) -> BufferSettings {
        self.buffer_settings
    }
}

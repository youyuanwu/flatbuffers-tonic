use tonic::codec::Codec;

// The reason to have wrapper struct is the inner struct has a lifetime
// and it makes tonic generated code not compile. Simple struct wrapper
// makes the type also explicit.
mod wrappers {
    use flatbuffers_tonic::OwnedFBCodecable;

    use crate::helloworld_gen::helloworld;

    pub struct OwnedHelloRequest(
        pub flatbuffers_util::ownedfb::OwnedFB<helloworld::HelloRequest<'static>>,
    );

    impl OwnedFBCodecable for OwnedHelloRequest {
        fn new_from_bytes(buf: bytes::Bytes) -> Result<Self, flatbuffers::InvalidFlatbuffer> {
            let owned =
                flatbuffers_util::ownedfb::OwnedFB::<helloworld::HelloRequest>::new_from_bytes(
                    buf,
                )?;
            Ok(Self(owned))
        }

        fn into_bytes(self) -> bytes::Bytes {
            self.0.into_bytes()
        }
    }
}

#[test]
fn codec_test() {
    let mut codec = flatbuffers_tonic::FlatBuffersCodec::<
        wrappers::OwnedHelloRequest,
        wrappers::OwnedHelloRequest,
    >::new();
    let mut _encoder = codec.encoder();
    let mut _decoder = codec.decoder();
}

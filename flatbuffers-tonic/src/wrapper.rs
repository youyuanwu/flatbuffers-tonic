/// trait for codec to deal with owned flatbuffer
/// All tonic flatbuffer wrappers needs to implement this.
pub trait OwnedFBCodecable {
    fn new_from_bytes(buf: bytes::Bytes) -> Result<Self, flatbuffers::InvalidFlatbuffer>
    where
        Self: Sized;

    fn into_bytes(self) -> bytes::Bytes;
}

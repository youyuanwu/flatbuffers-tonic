/// trait for codec to deal with owned flatbuffer
/// All tonic flatbuffer wrappers needs to implement this.
pub trait OwnedFBCodecable {
    fn new_boxed(buf: Box<[u8]>) -> Result<Self, flatbuffers::InvalidFlatbuffer>
    where
        Self: Sized;

    fn get_slice(&self) -> &[u8];
}

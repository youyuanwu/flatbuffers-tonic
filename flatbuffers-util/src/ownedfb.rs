use flatbuffers::{Follow, InvalidFlatbuffer, Verifiable};

/// Stores the owned bytes of the flatbuffer type
/// and can access the actual type.
pub struct OwnedFB<T> {
    buf: Vec<u8>,
    index: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> OwnedFB<T> {
    pub fn new<'a>(buf: &'a [u8]) -> Result<OwnedFB<T>, InvalidFlatbuffer>
    where
        T: Verifiable + Follow<'a> + 'a,
    {
        check_flatbuffer::<T>(buf, 0)?;
        Ok(unsafe { Self::new_from_vec_unchecked(buf.to_owned(), 0) })
    }

    /// # Safety
    /// Caller is responsible for verifying the buffer and align the type T.
    pub unsafe fn new_from_vec_unchecked(buf: Vec<u8>, index: usize) -> Self {
        Self {
            buf,
            index,
            _phantom: std::marker::PhantomData,
        }
    }

    /// # Safety
    /// Caller is responsible for verifying the buffer and align the type T.
    pub unsafe fn new_from_builder_collapse(pair: (Vec<u8>, usize)) -> Self {
        unsafe { Self::new_from_vec_unchecked(pair.0, pair.1) }
    }

    pub fn new_from_vec(buf: Vec<u8>, index: usize) -> Result<OwnedFB<T>, InvalidFlatbuffer>
    where
        T: Verifiable + Follow<'static> + 'static,
    {
        check_flatbuffer::<T>(&buf, index)?;

        Ok(unsafe { Self::new_from_vec_unchecked(buf, index) })
    }

    /// This in practice is not zero copy.
    /// For tonic, the bytes is always shared BytesMut, so it cannot be directly turned into vec.
    /// And in general Bytes have multiple chunks, and flatbuffer need contiguous memory.
    /// So this will make a copy in most cases.
    pub fn new_from_bytes(buf: bytes::Bytes) -> Result<OwnedFB<T>, InvalidFlatbuffer>
    where
        T: Verifiable + Follow<'static> + 'static,
    {
        match buf.try_into_mut() {
            // This is zero copy if the Bytes has the full ownership of the vec.
            Ok(bytes_mut) => Self::new_from_vec(bytes_mut.into(), 0),
            // This will make a copy of the bytes.
            Err(bytes) => {
                // This happens in tonic DecodeBuf because it is a shared BytesMut.
                // So in practice tonic forces us to make a copy here.
                // debug_assert!(false, "zero copy failed.");
                Self::new_from_vec(bytes.to_vec(), 0)
            }
        }
    }

    pub fn get_ref<'a>(&'a self) -> <T as Follow<'a>>::Inner
    where
        T: Follow<'a>,
    {
        // Safety: We have already verified the buffer in `new_owned_fb`.
        unsafe { get_ref_flatbuffer_unchecked::<'a, T>(&self.buf, self.index) }
    }

    pub fn get_slice(&self) -> &[u8] {
        &self.buf[self.index..]
    }

    /// This may be zero copy if the vec capacity equals to length and index is zero.
    pub fn into_bytes(self) -> bytes::Bytes {
        // This is zero copy if vec cap == len.
        debug_assert_eq!(self.buf.capacity(), self.buf.len());
        let full_bytes = bytes::Bytes::from(self.buf);
        // adjust the offset should be zero copy.
        full_bytes.slice(self.index..)
    }
}

/// Generic check.
pub fn check_flatbuffer<'a, T>(buf: &[u8], index: usize) -> Result<(), InvalidFlatbuffer>
where
    T: Verifiable + Follow<'a> + 'a,
{
    let opts = flatbuffers::VerifierOptions::default();
    let mut v = flatbuffers::Verifier::new(&opts, buf);
    <flatbuffers::ForwardsUOffset<T>>::run_verifier(&mut v, index)?;
    Ok(())
}

/// # Safety
/// Caller is responsible for verifying the buffer.
pub unsafe fn get_ref_flatbuffer_unchecked<'a, T>(buf: &'a [u8], index: usize) -> T::Inner
where
    T: Follow<'a> + 'a,
{
    // Safety: We have already verified the buffer in `check_flatbuffer`.
    unsafe { <flatbuffers::ForwardsUOffset<T>>::follow(buf, index) }
}

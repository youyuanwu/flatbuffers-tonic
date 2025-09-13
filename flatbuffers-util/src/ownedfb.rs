use flatbuffers::{Follow, InvalidFlatbuffer, Verifiable, root, root_unchecked};

/// Stores the owned bytes of the flatbuffer type
/// and can access the actual type.
pub struct OwnedFB<T> {
    buf: Box<[u8]>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> OwnedFB<T> {
    pub fn new<'a>(buf: &'a [u8]) -> OwnedFB<T>
    where
        T: Verifiable + Follow<'a> + 'a,
    {
        let _verified = root::<'a, T>(buf).expect("Invalid Flatbuffer");
        unsafe { Self::new_boxed_unchecked(buf.to_owned().into_boxed_slice()) }
    }

    /// # Safety
    /// Caller is responsible for verifying the buffer.
    pub unsafe fn new_boxed_unchecked(buf: Box<[u8]>) -> Self {
        Self {
            buf,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new_boxed(buf: Box<[u8]>) -> Result<OwnedFB<T>, InvalidFlatbuffer>
    where
        T: Verifiable + Follow<'static> + 'static,
    {
        check_flatbuffer::<T>(&buf)?;

        Ok(unsafe { Self::new_boxed_unchecked(buf) })
    }

    pub fn get_ref<'a>(&'a self) -> <T as Follow<'a>>::Inner
    where
        T: Follow<'a>,
    {
        // Safety: We have already verified the buffer in `new_owned_fb`.
        unsafe { root_unchecked::<'a, T>(&self.buf) }
    }

    pub fn get_slice(&self) -> &[u8] {
        &self.buf
    }
}

/// Generic check.
pub fn check_flatbuffer<'a, T>(buf: &[u8]) -> Result<(), InvalidFlatbuffer>
where
    T: Verifiable + Follow<'a> + 'a,
{
    // Verify buffer using unsafe code to handle lifetime issues
    // Safety: We drop the unsafe slice immediately.
    unsafe {
        // Transmute the slice to have a 'static lifetime for verification purposes
        let static_slice: &'static [u8] = std::mem::transmute(buf);
        let _verified = flatbuffers::root::<T>(static_slice)?;
    }
    Ok(())
}

/// # Safety
/// Caller is responsible for verifying the buffer.
pub unsafe fn get_ref_flatbuffer_unchecked<'a, T>(buf: &'a [u8]) -> T::Inner
where
    T: Verifiable + Follow<'a> + 'a,
{
    // Safety: We have already verified the buffer in `check_flatbuffer`.
    unsafe { root_unchecked::<'a, T>(buf) }
}

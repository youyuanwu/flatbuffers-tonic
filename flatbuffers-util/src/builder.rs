use flatbuffers::FlatBufferBuilder;

/// Wrapper of FlatBufferBuilder to provide OwnedFB creation.
/// This is to make type safe when using builder_collapse.
pub struct FBBuilder<T> {
    builder: FlatBufferBuilder<'static>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for FBBuilder<T> {
    fn default() -> Self {
        Self {
            builder: FlatBufferBuilder::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> FBBuilder<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get mutable reference to the builder.
    pub fn get_mut(&mut self) -> &mut FlatBufferBuilder<'static> {
        &mut self.builder
    }

    /// Finish the buffer and create OwnedFB.
    /// User still need to check the the root is created from this builder, otherwise there
    /// will be runtime error. (This is not marked unsafe, due to flatbuffers APIs are not
    /// marked as unsafe.)
    /// See issue: https://github.com/google/flatbuffers/issues/8698
    pub fn finish_owned(mut self, root: flatbuffers::WIPOffset<T>) -> crate::OwnedFB<T> {
        self.builder.finish_minimal(root);
        let (buf, index) = self.builder.collapse();
        unsafe { crate::OwnedFB::new_from_builder_collapse((buf, index)) }
    }
}

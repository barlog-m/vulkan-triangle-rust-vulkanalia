pub struct MappedMemory<'a> {
    allocator: &'a vulkanalia_vma::Allocator,
    allocation: vulkanalia_vma::Allocation,
    ptr: *mut u8,
}

impl<'a> MappedMemory<'a> {
    pub fn new(allocator: &'a vulkanalia_vma::Allocator, allocation: vulkanalia_vma::Allocation) -> Self {
        let ptr = unsafe { allocator.map_memory(allocation) }.expect("Failed to map buffer memory");
        assert!(!ptr.is_null());
        Self {
            allocator,
            allocation,
            ptr,
        }
    }

    /// Returns a mutable slice into the mapped memory. The pointer comes from a
    /// unique mapping owned by this guard; callers must not create aliasing
    /// slices for the same guard.
    #[allow(clippy::mut_from_ref)]
    pub fn as_slice_mut<T>(&self, len: usize) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr as *mut T, len) }
    }
}

impl Drop for MappedMemory<'_> {
    fn drop(&mut self) {
        unsafe {
            self.allocator.unmap_memory(self.allocation);
        }
    }
}

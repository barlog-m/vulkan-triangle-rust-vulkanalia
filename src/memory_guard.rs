pub struct MappedMemory<'a> {
    allocator: &'a vk_mem::Allocator,
    allocation: &'a mut vk_mem::Allocation,
    ptr: *mut u8,
}

impl<'a> MappedMemory<'a> {
    pub fn new(allocator: &'a vk_mem::Allocator, allocation: &'a mut vk_mem::Allocation) -> Self {
        let ptr = unsafe { allocator.map_memory(allocation) }.expect("Failed to map buffer memory");
        assert!(!ptr.is_null());
        Self {
            allocator,
            allocation,
            ptr,
        }
    }

    pub fn as_slice_mut<T>(&self, len: usize) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr as *mut T, len) }
    }
}

impl<'a> Drop for MappedMemory<'a> {
    fn drop(&mut self) {
        unsafe {
            self.allocator.unmap_memory(self.allocation);
        }
    }
}

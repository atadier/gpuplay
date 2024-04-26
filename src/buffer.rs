#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BufferUniforms {
    pub time: f32,
}

pub unsafe fn to_slice<T: Sized>(p: &T) -> &[u8] {
    std::slice::from_raw_parts((p as *const T) as *const u8, std::mem::size_of::<T>())
}

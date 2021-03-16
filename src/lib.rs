use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

#[derive(Clone)]
pub struct RelaxedBuffer {
    data:  Arc<[AtomicU32]>,
    width: u32,
    height: u32,
}
impl RelaxedBuffer{
    pub fn new(width: u32, height:u32, init_value: u32) -> Self {
        let size = width.checked_mul(height).unwrap() as usize;
        if size == 0{
            panic!("width and height should be > 0");
        }
        let mut vec = Vec::with_capacity(size);
        vec.resize_with(size, || AtomicU32::new(init_value));
        RelaxedBuffer {
                data: Arc::from(vec),
                width,
                height,
        }
    }
    pub fn get(&self, idx: usize) -> u32 {
        self.data[idx].load(Relaxed)
    }
    pub fn set(&self, idx: usize, val: u32) {
        self.data[idx].store(val, Relaxed);
    }
    pub fn fill(&self, val: u32){
        for idx in 0..self.data.len(){
            unsafe {
                self.data.get_unchecked(idx).store(val, Relaxed);
            }
        }
    }
    pub fn copy_into_slice<T>(&self, dest: &mut [T])
    where T: bytemuck::Pod {
            let target_u32: &mut [u32] = bytemuck::cast_slice_mut(dest);
            for idx in 0..self.data.len(){
                unsafe {
                    *target_u32.get_unchecked_mut(idx) = self.data.get_unchecked(idx).load(Relaxed);
                }
            }
    }
}
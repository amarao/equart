use std::sync::atomic::{AtomicU32};
use std::sync::atomic::Ordering::Relaxed;
pub struct Screen{
    pixels: Vec<std::sync::atomic::AtomicU32>,
    width: u32,
    height: u32
}

const BYTES_PER_PIXEL: usize = 4;
impl Screen{
    pub fn new(width: u32, height: u32) -> Result<Self, &'static str> {
        let size = (width  * height) as usize;
        if size % 8 !=0 {
            return Err("Width * height should be multiplier of 2.")
        }
        let mut draw_buffer = Screen{
            pixels: Vec::with_capacity(size),
            width,
            height
        };
        draw_buffer.pixels.resize_with(size, || AtomicU32::new(0xFFFFFFFF));
        Ok(draw_buffer)
    }
    
    /// Copy Screen content to u8 buffer
    /// 
    /// Buffer must be the same size as Screen.
    pub fn copy_to_buffer<T>(&self, target: &mut [T]) -> Result<(), &'static str>
    where T: bytemuck::Pod{
        if self.pixels.len() * BYTES_PER_PIXEL != target.len() * std::mem::size_of_val(&target[0]){
            return Err("Size of target should be the same as source.")
        }
        let target_u32: &mut [u32] = bytemuck::cast_slice_mut(target);
        for idx in 0..self.pixels.len(){
            target_u32[idx] = self.pixels[idx].load(Relaxed);
        }
        Ok(())
    }

    /// Fill whole Screen with specific color
    pub fn fill(&mut self, color: u32) {
        for b in &mut self.pixels{
            b.store(color, Relaxed);
        }
    }

    pub fn test(&self) -> u32{
        self.pixels[0].load(Relaxed)
    }
}
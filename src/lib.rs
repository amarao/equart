use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
use std::sync::Arc;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;



#[derive(Clone)]
pub struct RelaxedBuffer {
    data:  Arc<[AtomicU32]>,
}
impl RelaxedBuffer{
    pub fn new(size:usize, init_value: u32) -> Self {
        if size == 0{
            panic!("width and height should be > 0");
        }
        let mut vec = Vec::with_capacity(size);
        vec.resize_with(size, || AtomicU32::new(init_value));
        RelaxedBuffer {
                data: Arc::from(vec),
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

pub struct EasyScreen{
    buff: RelaxedBuffer,
    width: u32,
    height: u32,
    sdl_thread: std::thread::JoinHandle<()>,
}

impl Default for EasyScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl EasyScreen{
    pub fn new() -> Self{
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let sdl_thread = std::thread::spawn(move || {EasyScreen::display_thread(tx);});
        let (buff, width, height) = rx.recv().unwrap();
        EasyScreen{buff, width, height, sdl_thread}
    }

    fn display_thread(tx: std::sync::mpsc::SyncSender<(RelaxedBuffer, u32, u32)>){
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("title to be replace by a builder", 0, 0)
            .fullscreen_desktop()
            .borderless()
            .build()
            .unwrap();
        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .accelerated()
            .build()
            .unwrap();
        sdl_context.mouse().show_cursor(false);
        let (width, height) = canvas.output_size().unwrap();
        
        let texture_creator = canvas.texture_creator();
        let mut whole_screen = texture_creator
            .create_texture_streaming(
                sdl2::pixels::PixelFormatEnum::ABGR8888,
                width as u32,
                height as u32,
            )
            .unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        whole_screen.set_blend_mode(sdl2::render::BlendMode::None);

        let buff = RelaxedBuffer::new((width * height) as usize, 0);
        tx.send((buff.clone(), width, height)).unwrap();
        loop {
            whole_screen.with_lock(
                None,
                |bytearray, _|{
                    buff.copy_into_slice(bytearray);
                }
            ).unwrap();
            canvas.copy(&whole_screen, None, None).unwrap();
            canvas.present();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),
                    _ => {}
                }
            }
        }
    }

    pub fn fill(&self, color: u32){
        self.buff.fill(color);
    }

    pub fn width(&self)->u32 {self.width}
    pub fn height(&self)->u32 {self.height}

    /// Put pixel with coodinates wrapping,
    /// inverse location of (0,0) to be at
    /// lowest left corner.
    pub fn put_pixel(&self, x: u32, y: u32, color: u32){
        let true_x = x % self.width;
        let true_y = y % self.height;
        let inv_y = self.height - true_y - 1;
        let offset = (inv_y * self.width + true_x) as usize;
        self.buff.set(offset, color);
    }

    pub fn wait(&self){
        loop{};
    }
}
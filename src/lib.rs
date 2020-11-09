use image as im;
use piston_window;
use gfx_device_gl;
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError,sync_channel};
use std::thread::spawn;

pub type Buffer=im::ImageBuffer<im::Rgba<u8>,Vec<u8>>;

pub trait BufferExtentions{
    fn new(x:u32, y:u32)-> Self;
    fn scale(&self, new_x:u32, new_y:u32)-> Self;
    fn as_texture(
        &self,
        window: &mut piston_window::PistonWindow
    ) -> piston_window::Texture<gfx_device_gl::Resources>;
}

impl BufferExtentions for Buffer{
    fn new(x:u32, y:u32)-> Self{
        im::ImageBuffer::from_fn(x, y, |_, __| { im::Rgba([255,255,255,255]) })
    }
    
    fn scale(&self, new_x:u32, new_y:u32)-> Self{
        let old_x = self.width();
        let old_y = self.height();
        im::ImageBuffer::from_fn(new_x, new_y, |x, y| {
            if x < old_x && y < old_y {
                *(self.get_pixel(x, y))
            }else{
                im::Rgba([255,255,255,255])
            }
        })
    
    }

    fn as_texture(
        &self,
        window: &mut piston_window::PistonWindow
    ) -> piston_window::Texture<gfx_device_gl::Resources>
    {
        let mut texture_context = window.create_texture_context();
        piston_window::Texture::from_image(
                &mut texture_context,
                &self,
                &piston_window::TextureSettings::new()
            ).unwrap()
    }
}

#[derive(Debug)]
pub enum Command {
    NewResolution(u32, u32, SyncSender<Buffer>),
    NeedUpdate()
}

#[derive(Debug)]
struct PerThread {
    control_tx: SyncSender<Command>,
    draw_rx: Receiver<Buffer>,
    buf: Buffer
}

impl PerThread {
    fn new<C, T>(x: u32, y: u32, closure: C, id: usize) -> Self 
    where
        C: FnOnce(SyncSender<Buffer>,Receiver<Command>, usize) -> T,
        C: Send + 'static,
        T: Send + 'static
    {
        let (control_tx, control_rx): (SyncSender<Command>, Receiver<Command>) = sync_channel(1);
            let (draw_tx, draw_rx): (SyncSender<Buffer>, Receiver<Buffer>) = sync_channel(1);
        spawn(move ||{closure(draw_tx, control_rx, id)});
        Self{
            control_tx: control_tx,
            draw_rx: draw_rx,
            buf:Buffer::new(x, y)
        }
    }
    fn recieve_update(&mut self){
            match self.draw_rx.try_recv(){
                Ok(buf) =>{
                    self.buf=buf;
                }
                Err(TryRecvError::Empty) => {println!("update missed");}
                Err(TryRecvError::Disconnected) => {
                    println!("disconnected in draw");
                }
        }
    }
    fn request_update(&self){
        if let Err(err) =self.control_tx.try_send(Command::NeedUpdate()){
            println!("update request errorr: {}", err);
        }
    }

    fn texture(
        &self,
        window: &mut piston_window::PistonWindow
    ) -> piston_window::Texture<gfx_device_gl::Resources>{
        self.buf.as_texture(window)
    }

    fn resize(&mut self, new_x: u32, new_y: u32) -> Result<(), ()> {
        let (new_draw_tx, new_draw_rx): (SyncSender<Buffer>, Receiver<Buffer>) = sync_channel(1);
        if let Err(_) = self.control_tx.send(Command::NewResolution(
            new_x, new_y, new_draw_tx
        )){ return Err(())};
        self.draw_rx = new_draw_rx;
        self.buf = self.buf.scale(new_x, new_y);
        Ok(())
    }
    
}

pub struct Threads {
    cpus: usize,
    threads: Vec<PerThread>,
    x: u32,
    y: u32
}

type Texture = piston_window::Texture<gfx_device_gl::Resources>;

impl Threads {
    pub fn new<C, T>(x: u32, y: u32, closure: C) -> Self
    where 
        C: FnOnce(SyncSender<Buffer>, Receiver<Command>, usize) -> T,
        C: Send + 'static,
        C: Copy,
        T: Send + 'static
    {
        let cpus = num_cpus::get();
        let mut retval: Self = Self{
            cpus: cpus,
            threads: Vec::with_capacity(cpus),
            x: x,
            y: y
        };
        for cpu in 0..retval.cpus {
            retval.threads.push(PerThread::new(x, y/retval.cpus as u32, closure, cpu));
        }
        retval
    }

    pub fn request_update(&self){
        for cpu in 0..self.cpus {
            self.threads[cpu].request_update();
        }
    }

    pub fn recieve_update(&mut self){
        for cpu in 0..self.cpus {
            self.threads[cpu].recieve_update();
        }
    }

    pub fn get_textures(&self, window: &mut piston_window::PistonWindow) -> Vec<Texture>{
        let mut textures: Vec<piston_window::Texture<gfx_device_gl::Resources>> = Vec::with_capacity(self.cpus);
        for cpu in 0..self.cpus {
            textures.push(self.threads[cpu].texture(window));
        }
        textures
    }

    pub fn resize (&mut self, mut x: u32, mut y: u32){
        if x < 16 || y < 16 {
            println!("New resolution is too low {}x{}", x, y);
            x = std::cmp::max(x, 16);
            y = std::cmp::max(y, 16);
        }
        println!("Resize event, from {}x{} to {}x{}.", self.x, self.y, x, y);
        for cpu in 0..self.cpus{
            if self.threads[cpu].resize(x, y/self.cpus as u32) == Err(()){
                println!("Unable to resize");
                return;
            }
        }
        self.x = x;
        self.y = y;
    }

    
}

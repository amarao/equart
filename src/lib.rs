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
pub struct Thread {
    control_tx: SyncSender<Command>,
    draw_rx: Receiver<Buffer>,
    buf: Buffer
}

impl Thread {
    pub fn new<C, T>(x: u32, y: u32, closure: C) -> Self 
    where
        C: FnOnce(SyncSender<Buffer>,Receiver<Command>) -> T,
        C: Send + 'static,
        T: Send + 'static
    {
        let (control_tx, control_rx): (SyncSender<Command>, Receiver<Command>) = sync_channel(1);
            let (draw_tx, draw_rx): (SyncSender<Buffer>, Receiver<Buffer>) = sync_channel(1);
        
        spawn(||{closure(draw_tx, control_rx);});
        Self{
            control_tx: control_tx,
            draw_rx: draw_rx,
            buf:Buffer::new(x, y)
        }
    }
    pub fn recieve_update(&mut self){
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
    pub fn request_update(&self){
        if let Err(err) =self.control_tx.try_send(Command::NeedUpdate()){
            println!("update request errorr: {}", err);
        }
    }

    pub fn texture(
        &self,
        window: &mut piston_window::PistonWindow
    ) -> piston_window::Texture<gfx_device_gl::Resources>{
        self.buf.as_texture(window)
    }

    pub fn resize(&mut self, new_x: u32, new_y: u32) -> Result<(), ()> {
        let (new_draw_tx, new_draw_rx): (SyncSender<Buffer>, Receiver<Buffer>) = sync_channel(1);
        if let Err(_) = self.control_tx.send(Command::NewResolution(
            new_x, new_y, new_draw_tx
        )){ return Err(())};
        self.draw_rx = new_draw_rx;
        self.buf = self.buf.scale(new_x, new_y);
        Ok(())
    }
    
}
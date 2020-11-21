use image as im;
use piston_window;
use gfx_device_gl;
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError,sync_channel};
use std::thread;

pub type Buffer=im::ImageBuffer<im::Rgba<u8>,Vec<u8>>;

pub trait BufferExtentions{
    fn new(x:u32, y:u32)-> Self;
    fn scale(&self, new_x:u32, new_y:u32)-> Self;
    fn as_texture(
        &self,
        texture_context: &mut piston_window::TextureContext<gfx_device_gl::Factory, gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>
    ) -> piston_window::Texture<gfx_device_gl::Resources>;
}

impl BufferExtentions for Buffer{
    fn new(x:u32, y:u32)-> Self{
        im::ImageBuffer::from_fn(x, y, |_, __| { im::Rgba([255,255,255,255]) })
    }
    
    fn scale(&self, new_x:u32, new_y:u32)-> Self{
        let old_y = self.height();
        let old_x = self.width();
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
        texture_context: &mut piston_window::TextureContext<gfx_device_gl::Factory, gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>
    ) -> piston_window::Texture<gfx_device_gl::Resources>
    {
        piston_window::Texture::from_image(
                texture_context,
                &self,
                &piston_window::TextureSettings::new()
            ).expect("Can't make texture.")
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
    buf: Buffer,
    pub span: f64
}

impl PerThread {
    fn new<F, T>(x: u32, y: u32, id: usize, span: f64, f: F) -> Self 
    where
        F: FnOnce(usize) -> T,
        F: Send + 'static + Copy,
        T: DrawingApp
    {
        let (control_tx, control_rx): (SyncSender<Command>, Receiver<Command>) = sync_channel(1);
            let (draw_tx, draw_rx): (SyncSender<Buffer>, Receiver<Buffer>) = sync_channel(2);
        let thread_name = format!("thread {}", id);
        thread::Builder::new().name(thread_name).spawn(
            move ||{
                println!("Spawned thread for cpu {}", id);
                thread_worker(
                    draw_tx,
                    control_rx,
                    x,
                    y,
                    id,
                    f(id)
                )
            }
        ).unwrap();
        Self{
            control_tx: control_tx,
            draw_rx: draw_rx,
            buf:Buffer::new(x, y),
            span: span
        }
    }

    fn recieve_update(&mut self) -> Result<(), ()>{
        match self.draw_rx.try_recv(){
            Ok(buf) =>{
                self.buf=buf;
            }
            Err(TryRecvError::Empty) => {println!("update missed.");}
            Err(TryRecvError::Disconnected) => {
                println!("Thread terminated!");
                return Err(());
            }
        }
        Ok(())
    }
    fn request_update(&self){
        if let Err(err) =self.control_tx.try_send(Command::NeedUpdate()){
            println!("update request errorr: {}", err);
        }
    }

    fn texture(
        &self,
        texture_context: &mut piston_window::TextureContext<gfx_device_gl::Factory, gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>
    ) -> piston_window::Texture<gfx_device_gl::Resources>{
        self.buf.as_texture(texture_context)
    }

    fn resize(&mut self, new_x: u32, new_y: u32) -> Result<(), ()> {
        let (new_draw_tx, new_draw_rx): (SyncSender<Buffer>, Receiver<Buffer>) = sync_channel(2);
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
    y: u32,
}

type Texture = piston_window::Texture<gfx_device_gl::Resources>;

pub struct TextureIterator<'a> {
    threads_iter:std::slice::Iter<'a, PerThread>,
    texture_context: &'a mut piston_window::TextureContext<gfx_device_gl::Factory, gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>
}

pub struct TextureData {
    pub texture: Texture,
    pub span: f64
}
fn span(cpu: usize, cpus: usize) -> f64 {
    cpu as f64 / cpus as f64
}


impl Threads {

    pub fn new<F, T>(x: u32, y: u32, cpus: usize, f: F) -> Self
    where 
        F: Fn(usize) -> T,
        F: Send + 'static + Copy,
        T: DrawingApp
    {
        let mut retval: Self = Self{
            cpus: cpus,
            threads: Vec::with_capacity(cpus),
            x: x,
            y: y,
        };
        for cpu in 0..retval.cpus {
            retval.threads.push(PerThread::new(
                x,
                y/retval.cpus as u32,
                cpu,
                span(cpu, cpus),
                f,
            ))
        }
        retval
    }

    pub fn request_update(&self){
        for thread in &self.threads {
            thread.request_update();
        }
    }

    pub fn recieve_update(&mut self){
        for cpu in 0..self.cpus {
            if let Err(_) = self.threads[cpu].recieve_update(){
                println!("removing thread for cpu {}.", cpu);
                self.threads.remove(cpu);

                self.cpus -= 1;
                println!("{} threads left", self.cpus);
                break;
            }
        }
    }

    pub fn get_textures(&self, mut texture_context: &mut piston_window::TextureContext<gfx_device_gl::Factory, gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>) -> Vec<Texture>{
        let mut textures: Vec<piston_window::Texture<gfx_device_gl::Resources>> = Vec::with_capacity(self.cpus);
        for thread in &self.threads {
            textures.push(thread.texture(& mut texture_context));
        }
        textures
    }
    pub fn textures_iter<'a>(&'a self, texture_context: &'a mut piston_window::TextureContext<gfx_device_gl::Factory, gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>) -> TextureIterator {
        TextureIterator{
            threads_iter: self.threads.iter(),
            texture_context
        }
    }
    
    pub fn resize (&mut self, mut x: u32, mut y: u32){
        if x < 16 || y < 16 {
            println!("New resolution is too low {}x{}", x, y);
            x = std::cmp::max(x, 16);
            y = std::cmp::max(y, 16);
        }
        println!("Resize event, from {}x{} to {}x{}.", self.x, self.y, x, y);
        for thread in &mut self.threads {
            if thread.resize(x, y/self.cpus as u32) == Err(()){
                println!("Unable to resize");
                return;
            }
        }
        self.x = x;
        self.y = y;
    }

}

impl<'a> Iterator for TextureIterator <'a> {
    type Item = TextureData;
    fn next(&mut self) -> Option<Self::Item>{
        match self.threads_iter.next() {
            None => None,
            Some(thread) => Some(TextureData{
                texture: thread.texture(self.texture_context),
                span: thread.span
            })
        }
    }
}

struct ThreadWorkerState<A>{
    line: u32,
    app: A
}
impl<A> ThreadWorkerState<A>
    where A: DrawingApp
{
    fn new(app: A) -> Self {
        Self{
            line: 0,
            app: app
        }
    }
    fn draw(&mut self, buf: & mut Buffer) -> u32 {
        if self.line >= buf.height(){
            self.line = 0;
        }
        let y = self.line;
        for x in 0..buf.width(){
            buf.put_pixel(x, y, self.app.calculate_pixel(x, y))
            
        }
        self.line +=1;
        buf.width()
}
}


pub trait DrawingApp {
    fn new(id: usize)->Self;
    fn calculate_pixel(&mut self, x: u32, y: u32) -> im::Rgba<u8>;
}

pub fn thread_worker<A>(mut draw_tx: SyncSender<Buffer>, command: Receiver<Command>, x:u32, y:u32, id: usize, app: A)
where A: DrawingApp
{
    let mut sec_cnt: u32 = 0;
    let mut start = std::time::Instant::now();
    let mut state  = ThreadWorkerState::new(app);
    println!("new thread {}: {}x{}", id, x, y);
    let mut buf = Buffer::new(x, y);
    loop {
        match command.try_recv() {
            Ok(Command::NeedUpdate()) => {
                if let Err(_) = draw_tx.send(buf.clone()){
                    // must not print here, may be executed at shutdown
                    continue;
                }
                if start.elapsed().as_secs() >= 1 {
                    println!("thread {} rate: {:.2} Mpps", id, sec_cnt as f64 / start.elapsed().as_secs_f64()/1000.0/1000.0);
                    start = std::time::Instant::now();
                    sec_cnt = 0;
                }
            }
            Ok(Command::NewResolution(new_x, new_y, new_draw_tx)) => {
                println!("new thread {} resolution:{}x{}", id, new_x, new_y);
                buf = buf.scale(new_x, new_y);
                draw_tx = new_draw_tx;
            },
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                break;
            },
            Err(_) => {},
        }
        sec_cnt += state.draw(&mut buf);
    }
}
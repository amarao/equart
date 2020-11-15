use image as im;
use piston_window as pw;
use piston;
use std::sync::mpsc::{SyncSender, Receiver};
use equart::{BufferExtentions, Buffer, Command, Threads};

const DEFAULT_X: u32 = 1900;
const DEFAULT_Y: u32 = 1024;

fn main() {
    // let cpus = num_cpus::get();
    let cpus = 3;
    let mut start = std::time::Instant::now();
    let mut frames: u64 = 0;

    let mut window: piston_window::PistonWindow = match 
        pw::WindowSettings::new("equart", (DEFAULT_X, DEFAULT_Y))
        .exit_on_esc(true)
        .build() {
            Ok(window) => window,
            Err(err) => {
                println!("Unable to create a window: {}", err);
                return;
            }
        };

    let mut control = Threads::new(
        DEFAULT_X, DEFAULT_Y, cpus,
        move |draw_tx, control_rx, cpu|{
            println!("Spawning thread for cpu {}", cpu);
            thread_worker(draw_tx, control_rx, DEFAULT_X, DEFAULT_Y/cpus as u32, cpu)
        }
    );
    control.request_update();
    

    let mut events = pw::Events::new(
        (||{
            let mut settings = pw::EventSettings::new();
            settings.ups = 60;
            settings.max_fps = 60;
            settings
        })()
    );

    while let Some(e) = events.next(&mut window) {
        match e{
            piston::Event::Loop(piston::Loop::Idle(_)) => {},
            piston::Event::Loop(piston::Loop::AfterRender(_)) => {
                control.request_update();
                if start.elapsed().as_secs() > 0{
                    println!("Framerate: {:1}", frames as f32 / start.elapsed().as_secs_f32());
                    start = std::time::Instant::now();
                    frames = 0;
                }
            }
            piston::Event::Loop(piston::Loop::Render(_)) => {
                let mut texture_context = window.create_texture_context();
                let textures = control.textures_iter(& mut texture_context);
                window.draw_2d(
                    &e,
                    |context, graph_2d, _device| {
                        let mut transform = context.transform;
                        for texture_data in textures {
                            transform[1][2] = 1.0 - 2.0 * texture_data.span;
                            pw::image(
                                &texture_data.texture,
                                transform,
                                graph_2d
                            );
                        }
                    }
                );
                frames +=1;
            }
            
            piston::Event::Loop(piston::Loop::Update(_)) => {
                control.recieve_update();
            }
            piston::Event::Input(piston::Input::Resize(piston::ResizeArgs{window_size:_, draw_size:[new_x, new_y]}), _) => {
                control.resize(new_x, new_y);
            },
            piston::Event::Input(_, _) => {
            },
            ref something => {
                println!("Unexpected something: {:?}", something);
            },
        }
        window.event(&e);
    }
}

trait DrawingApp {
    fn new(id: usize)->Self;
    fn calculate_pixel(&mut self, x: u32, y: u32) -> im::Rgba<u8>;
}

struct DrawState{
    factor: u64,
    color: [u8;3]
}

impl DrawingApp for DrawState{
    fn new(id: usize)->Self {
        let color_bases = [
            [255, 0, 0],
            [255, 0,255],
            [0, 255,255],
            [255, 255, 0],
            [0, 255, 0],
            [0, 0,255],
            [255, 255, 255]
        ];
        Self{
            factor: 0xFEFABABE,
            color: color_bases[id % color_bases.len()]
        }
    }
    fn calculate_pixel(&mut self, x: u32, y: u32) -> im::Rgba<u8> {
        self.factor ^= self.factor << 13;
        self.factor ^= self.factor >> 17;
        self.factor ^= self.factor << 5;
        let rnd = self.factor.to_be_bytes()[0];
        im::Rgba([
            rnd & self.color[0],
            rnd & self.color[1],
            rnd & self.color[2],
            rnd,
        ])
    }
}

struct ThreadWorkerState<A>{
    line: u32,
    app: A
}
impl<A> ThreadWorkerState<A>
    where A: DrawingApp
{
    fn new(id: usize) -> Self {
        Self{
            line: 0,
            app: A::new(id)
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


fn thread_worker(mut draw_tx: SyncSender<equart::Buffer>, command: Receiver<Command>, x:u32, y:u32, id: usize){
    let mut sec_cnt: u32 = 0;
    let mut start = std::time::Instant::now();
    
    println!("new thread {}: {}x{}", id, x, y);
    let mut state: ThreadWorkerState<DrawState> = ThreadWorkerState::new(id);
    let mut buf = equart::Buffer::new(x, y);
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

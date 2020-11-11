use image as im;
use piston_window as pw;
use piston;
use std::sync::mpsc::{SyncSender, Receiver};
use equart::{BufferExtentions, Command, Threads};

const DEFAULT_X: u32 = 1900;
const DEFAULT_Y: u32 = 1024;

fn main() {
    let cpus = num_cpus::get();

    let mut control = Threads::new(DEFAULT_X, DEFAULT_Y, 
        move |draw_tx, control_rx, cpu|{
            println!("Spawning thread for cpu {}", cpu);
            calc(draw_tx, control_rx, DEFAULT_X, DEFAULT_Y/cpus as u32, cpu)
        }
    );
    
    let mut window = match 
        pw::WindowSettings::new("equart", (DEFAULT_X, DEFAULT_Y))
        .exit_on_esc(true)
        .build() {
            Ok(window) => window,
            Err(err) => {
                drop(control);
                println!("Unable to create a window: {}", err);
                return;
            }
        };

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
            }
            piston::Event::Loop(piston::Loop::Render(_)) => {
                let textures = control.textures_iter(& mut window);
                window.draw_2d(
                    &e,
                    |context, graph_2d, _device| {
                        let mut transform = context.transform;
                        for texture_data in textures {
                            transform[1][2] = 1.0 - 2.0 * texture_data.span ;
                            pw::image(
                                &texture_data.texture,
                                transform,
                                graph_2d
                            );
                        }
                    }
                );
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

fn calc(mut draw: SyncSender<equart::Buffer>, command: Receiver<Command>, mut x:u32, mut y:u32, id: usize){
    let mut sec_cnt: u64 = 0;
    let mut factor: u64 = 0xFEFABABE;
    let mut start = std::time::Instant::now();
    
    println!("new thread {}: {}x{}", id, x, y);
    let mut j = 0;
    let color_bases = [
        [255, 0, 0],
        [255, 0,255],
        [0, 255,255],
        [255, 255, 0],
        [0, 255, 0],
        [0, 0,255],
        [255, 255, 255]
    ];
    let color_base = color_bases[id % color_bases.len()];
    let mut buf = equart::Buffer::new(x, y);
    loop{
        match command.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // must not print here, may be executed at shutdown
                return;
            },
            Ok(Command::NewResolution(new_x, new_y, new_draw)) => {
                    println!("new thread resolution:{}x{}", new_x, new_y);
                    x = new_x;
                    y = new_y;
                    buf = buf.scale(x, y);
                    draw = new_draw;
            },
            Ok(Command::NeedUpdate()) => {
                if let Err(_) = draw.send(buf.clone()){
                    // must not print here, may be executed at shutdown
                    continue;
                }
                if start.elapsed().as_secs() >= 1 {
                    println!("thread rate: {:.2} Mpps", sec_cnt as f64 / start.elapsed().as_secs_f64()/1000.0/1000.0);
                    start = std::time::Instant::now();
                    sec_cnt = 0;
                }
            },
            Err(_empty) => {
                if j >= y {
                    j = 0;
                };
                for i in 0..x {
                    sec_cnt +=1;
                    factor ^= factor << 13;
                    factor ^= factor >> 17;
                    factor ^= factor << 5;
                    let rnd = factor.to_be_bytes()[0];
                    buf.put_pixel(
                        i,
                        j,
                        im::Rgba([
                            rnd & color_base[0],
                            rnd & color_base[1],
                            rnd & color_base[2],
                            rnd,

                        ])
                    );
                };
                j += 1;
            }
        }
    }
}

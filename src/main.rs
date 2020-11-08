use image as im;
use piston_window as pw;
use piston;
use std::sync::mpsc::{SyncSender, Receiver};
use equart::{BufferExtentions,Command,Thread};

fn main() {
    let mut x = 800;
    let mut y  = 600;
    let cpus = num_cpus::get();
    let color_bases = [
        [255, 0, 0],
        [0, 255, 0],
        [255, 255, 0],
        [0, 0,255],
        [255, 0,255],
        [0, 255,255],
        [255, 255, 255]
    ];
    let mut control:Vec<Thread> = Vec::with_capacity(cpus);

    for cpu in 0..cpus{
        control.push(
            Thread::new(
                x,
                y/cpus as u32,
                move |draw_tx, control_rx|{
                    println!("Spawning thread for cpu {}", cpu);
                    calc(draw_tx, control_rx, x, y/cpus as u32, color_bases[cpu])
            }
        ));
    }
    let mut window = match 
        pw::WindowSettings::new("equart", (x, y))
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
                for cpu in 0..cpus{
                    control[cpu].request_update();
                }
            }
            piston::Event::Loop(piston::Loop::Render(_)) => {
                let mut textures: Vec<piston_window::Texture<gfx_device_gl::Resources>> = Vec::new();
                for cpu in 0..cpus {
                    textures.push(control[cpu].texture(&mut window));
                }
                window.draw_2d(
                    &e,
                    |context, graph_2d, _device| {
                        let mut transform = context.transform;
                        for cpu in 0..cpus {
                            transform[1][2] = 1.0 - 2.0 * cpu as f64 / cpus as f64 ;
                            pw::image(
                                &textures[cpu],
                                transform,
                                graph_2d
                            );
                        }
                    }
                );
                for _ in 0..cpus{
                    drop(textures.pop());
                }
                drop(textures);
            }
            piston::Event::Loop(piston::Loop::Update(_)) => {
                for cpu in 0..cpus {
                    control[cpu].recieve_update();
                }
            }
            piston::Event::Input(piston::Input::Resize(piston::ResizeArgs{window_size:_, draw_size:[mut new_x, mut new_y]}), _) => {
                if new_x < 16 || new_y < 16 {
                    println!("New resolution is too low {}x{}", new_x, new_y);
                    new_x = std::cmp::max(new_x, 16);
                    new_y = std::cmp::max(new_y, 16);
                }
                println!("Resize event: {}x{} (was {}x{})", new_x, new_y, x, y);
                for cpu in 0..cpus{
                    if control[cpu].resize(new_x, new_y/cpus as u32) == Err(()){
                        println!("Unable to resize");
                        return;
                    }
                }
                x = new_x;
                y = new_y;
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

fn calc(mut draw: SyncSender<equart::Buffer>, command: Receiver<Command>, max_x:u32, max_y:u32, color_base:[u8;3]){
    let mut cur_x = max_x;
    let mut cur_y = max_y;
    let mut cnt: u64 = 0;
    let mut start = std::time::Instant::now();
    println!("new thread: {}x{}", max_x, max_y);
    let mut buf = equart::Buffer::new(max_x, max_y);
    loop{
        match command.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // must not print here, may be executed at shutdown
                return;
            },
            Ok(Command::NewResolution(new_x, new_y, new_draw)) => {
                    println!("new thread resolution:{}x{}", new_x, new_y);
                    cur_x = new_x;
                    cur_y = new_y;
                    buf = buf.scale(new_x, new_y);
                    draw = new_draw;
            },
            Ok(Command::NeedUpdate()) => {
                if let Err(_) = draw.send(buf.clone()){
                    // must not print here, may be executed at shutdown
                    continue;
                }
                if start.elapsed().as_secs() >= 1 {
                    println!("thread rate: {:.2} Mpps", cnt as f64 / start.elapsed().as_secs_f64()/1000.0/1000.0);
                    start = std::time::Instant::now();
                    cnt = 0;
                }
            }
            Err(_empty) => {
                for _ in 0..1000 {
                    cnt += 1;
                    buf.put_pixel(
                        (cnt % cur_x as u64) as u32,
                        (cnt % cur_y as u64) as u32,
                        im::Rgba([
                            if color_base[0] > 0 { (cnt % color_base[0] as u64) as u8 } else {0},
                            if color_base[1] > 0 { (cnt % color_base[1] as u64) as u8 } else {0},
                            if color_base[2] > 0 { (cnt % color_base[2] as u64) as u8 } else {0},
                            128,

                        ])
                    );
                 }
            }
        }
    }
}

use image as im;
use equart::{Threads, DrawingApp};

const DEFAULT_X: u32 = 1900;
const DEFAULT_Y: u32 = 1024;

struct App {
    window: piston_window::PistonWindow,
    events: piston_window::Events,
    control: Threads,
    cpus: usize,
    start: std::time::Instant,
    request_update_time: std::time::Duration,
    recieve_time: std::time::Duration,
    draw_time: std::time::Duration,
    other_time: std::time::Duration,
    frames: u64
}

impl App {
    fn new<U>(title: &str, cpus: usize, start_x: u32, start_y: u32) -> Self 
        where
            U: DrawingApp + Copy + 'static
        {
        let window: piston_window::PistonWindow = 
            piston_window::WindowSettings::new(title, (start_x, start_y))
            .exit_on_esc(true)
            .build().expect("Unable to create window");
        let mut settings = piston_window::EventSettings::new();
        settings.ups = 60;
        settings.max_fps = 60;
        let  events = piston_window::Events::new(settings);
        let control = Threads::new (DEFAULT_X, DEFAULT_Y, cpus, U::new);
        let zero = std::time::Duration::new(0, 0);
        Self {
            window,
            events,
            control,
            cpus,
            start: std::time::Instant::now(),
            request_update_time: zero,
            recieve_time: zero,
            draw_time: zero,
            other_time: zero,
            frames: 0
        }
        
    }
    fn next_event(&mut self) -> Option<piston::Event> {
        self.events.next(& mut self.window)
    }

    fn after_render(&mut self){
        let request_start = std::time::Instant::now();
        self.control.request_update();
        self.request_update_time += request_start.elapsed();
        if self.start.elapsed().as_secs() > 0{
            let elapsed = self.start.elapsed().as_secs_f32();
            println!(
                "FPS: {:.1}, req_time: {:.5}, recv_time {:.5}, draw_time: {:.5}, other: {:.5}",
                self.frames as f32 / elapsed,
                self.request_update_time.as_secs_f32()/elapsed,
                self.recieve_time.as_secs_f32()/elapsed,
                self.draw_time.as_secs_f32()/elapsed,
                self.other_time.as_secs_f32()/elapsed
            );
            self.start = std::time::Instant::now();
            self.frames = 0;
            self.request_update_time = std::time::Duration::new(0,0);
            self.recieve_time = std::time::Duration::new(0,0);
            self.draw_time = std::time::Duration::new(0,0);
            self.other_time = std::time::Duration::new(0,0);
        }
    }

    fn render(&mut self, e: &piston::Event){
        let draw_start = std::time::Instant::now();
        let mut texture_context = self.window.create_texture_context();
        let textures = self.control.textures_iter(& mut texture_context);
        self.window.draw_2d(
            e,
            |context, graph_2d, _device| {
                let mut transform = context.transform;
                for texture_data in textures {
                    transform[1][2] = 1.0 - 2.0 * texture_data.span;
                    piston_window::image(
                        &texture_data.texture,
                        transform,
                        graph_2d
                    );
                }
            }
        );
        self.frames +=1;
        self.draw_time += draw_start.elapsed();
    }

    fn update(&mut self){
        let recieve_start = std::time::Instant::now();
        self.control.recieve_update();
        self.recieve_time += recieve_start.elapsed();
    }

    fn input (&mut self, e: piston::Event) {
        if let piston::Event::Input(piston::Input::Resize(piston::ResizeArgs{window_size:_, draw_size:[new_x, new_y]}), _) = e {
            self.control.resize(new_x, new_y);
        }
    }

    fn finish_event(&mut self, e: piston::Event) {
        let other_start = std::time::Instant::now();
        self.window.event(&e);
        self.other_time += other_start.elapsed();
    }

}

fn main() {
    // let cpus = num_cpus::get();
    let cpus = 3;
    let mut start = std::time::Instant::now();
    let mut request_update_time = std::time::Duration::new(0,0);
    let mut recieve_time = std::time::Duration::new(0,0);
    let mut draw_time = std::time::Duration::new(0,0);
    let mut other_time = std::time::Duration::new(0,0);
    let mut frames: u64 = 0;

    let mut window: piston_window::PistonWindow = match 
        piston_window::WindowSettings::new("equart", (DEFAULT_X, DEFAULT_Y))
        .exit_on_esc(true)
        .build() {
            Ok(window) => window,
            Err(err) => {
                println!("Unable to create a window: {}", err);
                return;
            }
        };
    

    let mut control = Threads::new (DEFAULT_X, DEFAULT_Y, cpus, RandDraw::new);
    control.request_update();
    
    let mut settings = piston_window::EventSettings::new();
    settings.ups = 60;
    settings.max_fps = 60;
    let mut events = piston_window::Events::new(settings);

    while let Some(e) = events.next(&mut window) {
        match e{
            piston::Event::Loop(piston::Loop::Idle(_)) => {},
            piston::Event::Loop(piston::Loop::AfterRender(_)) => {
                let request_start = std::time::Instant::now();
                control.request_update();
                request_update_time += request_start.elapsed();
                if start.elapsed().as_secs() > 0{
                    let elapsed = start.elapsed().as_secs_f32();
                    println!(
                        "FPS: {:.1}, req_time: {:.5}, recv_time {:.5}, draw_time: {:.5}, other: {:.5}",
                        frames as f32 / elapsed,
                        request_update_time.as_secs_f32()/elapsed,
                        recieve_time.as_secs_f32()/elapsed,
                        draw_time.as_secs_f32()/elapsed,
                        other_time.as_secs_f32()/elapsed
                    );
                    start = std::time::Instant::now();
                    frames = 0;
                    request_update_time = std::time::Duration::new(0,0);
                    recieve_time = std::time::Duration::new(0,0);
                    draw_time = std::time::Duration::new(0,0);
                    other_time = std::time::Duration::new(0,0);
                }
            }
            piston::Event::Loop(piston::Loop::Render(_)) => {
                let draw_start = std::time::Instant::now();
                let mut texture_context = window.create_texture_context();
                let textures = control.textures_iter(& mut texture_context);
                window.draw_2d(
                    &e,
                    |context, graph_2d, _device| {
                        let mut transform = context.transform;
                        for texture_data in textures {
                            transform[1][2] = 1.0 - 2.0 * texture_data.span;
                            piston_window::image(
                                &texture_data.texture,
                                transform,
                                graph_2d
                            );
                        }
                    }
                );
                frames +=1;
                draw_time += draw_start.elapsed();
            }
            
            piston::Event::Loop(piston::Loop::Update(_)) => {
                let recieve_start = std::time::Instant::now();
                control.recieve_update();
                recieve_time += recieve_start.elapsed();
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
        let other_start = std::time::Instant::now();
        window.event(&e);
        other_time += other_start.elapsed();
    }
}

struct RandDraw{
    factor: u64,
    color: [u8;3]
}

impl DrawingApp for RandDraw{
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
    fn calculate_pixel(&mut self, _x: u32, _y: u32) -> im::Rgba<u8> {
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
use image as im;
use equart::{Threads, DrawingApp};
use piston::{Event, Loop};

const DEFAULT_X: u32 = 1900;
const DEFAULT_Y: u32 = 1024;

struct App {
    window: piston_window::PistonWindow,
    events: piston_window::Events,
    control: Threads,
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
            U: DrawingApp + 'static
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
            start: std::time::Instant::now(),
            request_update_time: zero,
            recieve_time: zero,
            draw_time: zero,
            other_time: zero,
            frames: 0
        }
        
    }
    fn next_event(&mut self) -> Option<Event> {
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

    fn render(&mut self, e: &Event){
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

    fn input (&mut self, i: &piston::input::Input) {
        if let piston::Input::Resize(piston::ResizeArgs{window_size:_, draw_size:[new_x, new_y]}) = i {
            self.control.resize(*new_x, *new_y);
        }
    }

    fn finish_event(&mut self, e: Event) {
        let other_start = std::time::Instant::now();
        self.window.event(&e);
        self.other_time += other_start.elapsed();
    }

}

fn main() {
    // let cpus = num_cpus::get();
    let cpus = 3;
    let mut app: App = App::new::<Equart>("equart", cpus, DEFAULT_X, DEFAULT_Y);

    while let Some(e) = app.next_event() {
        match e{
            Event::Loop(Loop::Idle(_)) => {},
            Event::Loop(Loop::AfterRender(_)) => app.after_render(),
            Event::Loop(Loop::Render(_)) => app.render(&e),
            Event::Loop(Loop::Update(_)) => app.update(),
            Event::Input(ref i, _) => app.input(&i),
            ref something => {
                println!("Unexpected something: {:?}", something);
            },
        }
        app.finish_event(e);
    }
}

struct RandDraw{
    factor: u64,
    color: [u8;3]
}

impl DrawingApp for RandDraw{
    fn new(id: usize, max_id: usize)->Self {
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
    fn resize(&mut self, old_x: u32, old_y: u32, new_x: u32, new_y: u32){

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


const WINDOW_X_START: f64 = -1.0;
const WINDOW_X_END: f64 = 1.0;
const WINDOW_Y_START: f64 = 1.0;
const WINDOW_Y_END: f64 = 1.0;


fn equart(x: f64, y:f64) -> f64{
    x.sin() - y
}

struct Equart {
    root_window_start_x: f64,
    root_window_start_y: f64,
    root_window_end_x: f64,
    root_window_end_y: f64,
    roots_map: Vec<Vec<f64>>,
    fixel_size: f64, //pixel size for float window
}

impl DrawingApp for Equart{
    fn new(id: usize, max_id: usize)->Self {
        Self{
            root_window_start_x: 0.0,
            root_window_start_y: 0.0,
            root_window_end_x: 1.0,
            root_window_end_y: 1.0,
            roots_map: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            fixel_size: 0.1
        }
    }
    fn resize(&mut self, old_x: u32, old_y: u32, new_x: u32, new_y: u32){

    }
    fn calculate_pixel(&mut self, _x: u32, _y: u32) -> im::Rgba<u8> {
        im::Rgba([
            0,
            1,
            2,
            255
        ])
    }
}

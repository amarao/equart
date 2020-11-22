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
        settings.ups = 120;
        settings.max_fps = 120;
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
    let cpus = num_cpus::get();
    // let cpus = 3;
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
    fn new(id: usize, max_id: usize, x: u32, y: u32)->Self {
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


const WINDOW_X_START: f64 = -8.0;
const WINDOW_X_END: f64 = 8.0;
const WINDOW_Y_START: f64 = -1.5;
const WINDOW_Y_END: f64 = 1.5;


fn equart(x: f64, y:f64) -> f64{
    x.sin() - y
}

struct Equart {  // per thread instance, each instance has own 'slice' to work with
    root_window_start_x: f64,
    root_window_end_x: f64,
    root_window_start_y: f64,
    root_window_end_y: f64,
    roots_map: Vec<Vec<f64>>,
    fixel_size_x: f64, //pixel size for float window
    fixel_size_y: f64, //pixel size for float window
    pixel_size_x: u32,
    pixel_size_y: u32
}

impl DrawingApp for Equart{
    fn new(id: usize, max_id: usize, x: u32, y: u32)->Self {
        let slice = Equart::slice(WINDOW_Y_START, WINDOW_Y_END, id, max_id);
        let mut value = Self{
            root_window_start_x: WINDOW_X_START,
            root_window_end_x: WINDOW_X_END,
            root_window_start_y: slice.0,
            root_window_end_y: slice.1,
            roots_map: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            fixel_size_x: 1.0,
            fixel_size_y: 1.0,
            pixel_size_x: x,
            pixel_size_y: y

        };
        value.update_fixel_size();
        value
        
    }
    fn resize(&mut self, old_x: u32, old_y: u32, new_x: u32, new_y: u32){
        self.pixel_size_x = new_x;
        self.pixel_size_y = new_y;
        self.update_fixel_size();
    }

    fn calculate_pixel(&mut self, x: u32, y: u32) -> im::Rgba<u8> {
        let matrix = self.pixel2matrix(x, y);
        match Equart::is_root(matrix, equart) {
            true => im::Rgba([0, 0, 0, 255]),
            false => im::Rgba([255 ,255, 255, 255]),
        }
    }
}

impl Equart{
    fn is_root<F>(matrix: Vec<[f64;2]>, f: F) -> bool 
    where
        F: Fn(f64, f64) -> f64
    {
        let mut zeroes: bool = false;
        let mut positive: bool = false;
        let mut negative: bool = false;
        for [x,y ] in matrix {
            let res = f(x, y);
            if res == 0.0 {
                zeroes = true;
            } else if res < 0.0 {
                negative = true;
            } else if res > 0.0 {
                positive = true;
            }
            // NaN is skippeed.

        }
        zeroes || (positive && negative)
    }

    fn pixel2matrix(&self, x: u32, y: u32)-> Vec<[f64;2]> {
        // only 2x2 fixel matrix
        let start_x = self.root_window_start_x + self.fixel_size_x * x as f64;
        let start_y = self.root_window_start_y + self.fixel_size_y * y as f64;
        vec![
            [start_x, start_y],
            [start_x + self.fixel_size_x, start_y],
            [start_x, start_y + self.fixel_size_y],
            [start_x + self.fixel_size_x, start_y + self.fixel_size_y],
        ]
    }
    fn slice(start: f64, end: f64, id: usize, max_id: usize) -> (f64, f64){
        let span = (end - start)/max_id as f64;
        let begin =  start + span * id as f64;
        let end = begin + span;
        (begin, end)
    }
    fn update_fixel_size(&mut self){
        self.fixel_size_x = (self.root_window_end_x - self.root_window_start_x)/self.pixel_size_x as f64;
        self.fixel_size_y = (self.root_window_end_y - self.root_window_start_y)/self.pixel_size_y as f64;
    }
}

#[cfg(test)]
mod equart_tests {
    use super::*;
    
    #[test]
    fn is_root_empty(){
        let data_in = vec![];
        assert_eq!(
            Equart::is_root(data_in, |_, __|{0.0}),
            false
        );
    }

    #[test]
    fn is_root_zerores(){
        let data_in = vec![[0.0, 0.0]];
        assert_eq!(
            Equart::is_root(data_in, |_, __|{0.0}),
            true
        );
    }

    #[test]
    fn is_root_positive(){
        let data_in = vec![[0.0, 0.0]];
        assert_eq!(
            Equart::is_root(data_in, |_, __|{1.0}),
            false
        );
    }
    
    #[test]
    fn is_root_negative(){
        let data_in = vec![[0.0, 0.0]];
        assert_eq!(
            Equart::is_root(data_in, |_, __|{-1.0}),
            false
        );
    }
    
    #[test]
    fn is_root_sign_change(){
        let data_in = vec![[-1.0, -1.0], [1.0, 1.0]];
        assert_eq!(
            Equart::is_root(data_in, |x, __|{x}),
            true
        );
    }


    #[test]
    fn slice_one(){
         assert_eq!(Equart::slice(0.0, 1.0, 0, 1), (0.0, 1.0));
    }
    
    #[test]
    fn slice_first_half(){
         assert_eq!(Equart::slice(0.0, 1.0, 0, 2), (0.0, 0.5));
    }
    
    #[test]
    fn slice_second_half(){
         assert_eq!(Equart::slice(0.0, 1.0, 1, 2), (0.5, 1.0));
    }
}
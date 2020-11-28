mod equart;
mod threads;

use image as im;
use threads::{App, DrawingApp};
use piston::{Event, Loop};

const DEFAULT_X: u32 = 1900;
const DEFAULT_Y: u32 = 1024;


fn main() {
    let cpus = num_cpus::get();
    // let cpus = 3;
    let mut app: App = App::new::<equart::Equart>("equart", cpus, DEFAULT_X, DEFAULT_Y);

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
    fn new(id: usize, _max_id: usize, _x: u32, _y: u32)->Self {
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


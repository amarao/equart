mod equart;
mod rand_draw;
mod threads;
mod fixel;

use threads::App;
use piston::{Event, Loop};

const DEFAULT_X: u32 = 1000;
const DEFAULT_Y: u32 = 1000;


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




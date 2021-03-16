use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use lib::RelaxedBuffer;

fn draw(buff: RelaxedBuffer){
    let mut c = 0;
    loop{
        c+=1;
        buff.fill(c);
    }
}
fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("equart", 0, 0)
        .fullscreen_desktop()
        .borderless()
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        // .present_vsync()
        .accelerated()
        .build()
        .unwrap();
    sdl_context.mouse().show_cursor(false);
    let (width, height) = canvas.output_size().unwrap();
    let mut screen = RelaxedBuffer::new(width, height, 0);
    let texture_creator = canvas.texture_creator();
    let mut whole_screen = texture_creator
        .create_texture_streaming(
            sdl2::pixels::PixelFormatEnum::ABGR8888,
            width as u32,
            height as u32,
        )
        .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    whole_screen.set_blend_mode(sdl2::render::BlendMode::None);
    let second_screen = screen.clone();
    std::thread::spawn(move ||draw(second_screen));
    loop {
            whole_screen.with_lock(
                None,
                |bytearray, _|{
                    screen.copy_into_slice(bytearray);
                }
            ).unwrap();
        canvas.copy(&whole_screen, None, None).unwrap();
        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                _ => {}
            }
        }
    }
}

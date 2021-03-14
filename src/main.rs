use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 noise demo", 0, 0)
        .fullscreen_desktop()
        .borderless()
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .unwrap();
    sdl_context.mouse().show_cursor(false);
    let (width, height) = canvas.output_size().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut whole_screen = texture_creator
        .create_texture_streaming(
            sdl2::pixels::PixelFormatEnum::ABGR8888,
            width as u32,
            height as u32,
        )
        .unwrap();
    whole_screen
        .update(
            //fill?
            None,
            &vec![0; (width * height * 4) as usize],
            (width * 4) as usize,
        )
        .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    whole_screen.set_blend_mode(sdl2::render::BlendMode::None);
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        canvas.copy(&whole_screen, None, None).unwrap();
        canvas.present();
    }
}

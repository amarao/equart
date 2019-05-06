extern crate minifb;

use std::time::{Duration, Instant};
use std::thread::sleep;
use minifb::{Key, WindowOptions, Window};

// trait Point {
//     fn point(&mut self, x:f64, y:f64);
// }
//
// impl Point for Turtle {
//     fn point(&mut self, x:f64, y:f64){
//
//         self.pen_up();
//         self.go_to([x, y]);
//         self.pen_down();
//         self.forward(1.0);
//         self.pen_up();
//     }
// }

struct Pixel<'a>{
    value: &'a u32,
    x:f64, //Cartesian coordinates
    y:f64,
    dx:f64, //Size of pixel in Cartesian
    dy:f64
}

struct Canvas {
    img: Vec<u32>,
    pixel_x: usize,
    pixel_y: usize,
    cartesian_x: f64,
    cartesian_y: f64,
    pixel_size_x: f64,
    pixel_size_y: f64,
    zero_x: usize,
    zero_y: usize,
    lattice_dim: usize,
}

impl Canvas{
    fn new(canvas_x: usize, canvas_y:usize, cartesian_x: f64, cartesian_y:f64, zero_position_x: usize, zero_position_y: usize, lattice_dim: usize) -> Canvas {
        let img_size = ((canvas_x as u64)*(canvas_y.1 as u64)) as usize;
        let canvas = Canvas{
                img: vec![0xFFFF_FFFF;img_size],
                pixel_x: canvas_x,
                pixel_y: canvas_y,
                zero_x: zero_position_x,
                zero_y: zero_position_y,
                cartesian_x: cartesian_x,
                cartesian_y: cartesian_y,
                pixel_size_x: cartesian_x/(canvas_x as f64),
                pixel_size_y: cartesian_y/(canvas_y as f64),
                lattice_dim:lattice_dim,
        };
        canvas
    }

}


impl Iterator for Canvas {
    type Item = Pixel;
    fn next(&mut self) -> Option<Pixel> {
        let value: &u32 = &self.img[0];
        let pixel = Pixel{value: value, x: 0.0, y:0.0, dx:0.0, dy:0.0};
        Some(pixel)
}


fn sign_change_on_lattice<F> (pixel:&Pixel, func: F, dim: u8) -> bool where
    F: Fn(f64, f64) -> f64
{
    let mut sign: Option<bool> = None;
    for i in 0..dim {
        for j in 0..dim{
            let x = pixel.x+pixel.dx/((dim - 1) as f64)*(i as f64);
            let y = pixel.y+pixel.dy/((dim - 1) as f64)*(j as f64);
            let res = func(x,y);
            if !res.is_finite() {continue};
            let num_sign:bool = res.signum() > 0.0;
            sign = match sign {
                None => {Some(num_sign)},
                Some(old_sign) if old_sign != num_sign => { return true },
                _ => {continue}
            };
        }
    }
    false
}

const DRAW_X: u32 = 1920;
const DRAW_Y: u32 = 1080;
const PAN: f64 = 25.0;
const FUNC_X: f64 = 1.92*PAN;
const FUNC_Y: f64 = 1.08*PAN;





fn show_and_wait(canvas:Canvas){ {

    let mut window = Window::new("Test - ESC to exit",
                                 canvas.pix_dim_x,
                                 canvas.pix_dim_y,
                                 WindowOptions::default()
                                 ).unwrap();

    std::thread::sleep(Duration::new(0,150_000_000));
    window.update_with_buffer(&canvas.img).unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let start = Instant::now();
        window.update();
        let spend = start.elapsed();
        sleep(Duration::new(0,1000000000/60).checked_sub(spend).unwrap_or(Duration::new(0,0)));
    }
}

}

fn main() {
    let lattice_size = 9u8;
    let mut canvas = Canvas::new((DRAW_X as usize, DRAW_Y as usize), (FUNC_X, FUNC_Y), None, lattice_size as usize);
    let eq = |x:f64, y:f64| (x*x).sin() - (y*y).cos();
    // let x_offset = (DRAW_X/2) as i32;
    // let y_offset = (DRAW_Y/2) as i32;
    for pixel in canvas.into_iter(){
        let sign_change = sign_change_on_lattice(&pixel, eq, lattice_size);
            //turtle.point(i as f64, j as f64);
        //canvas.dot(&pixel, eq(pixel.x, pixel.y));
    }
    //canvas.finish();
    //show_and_wait(canvas);
}

    //let mut turtle = Turtle::new();
    //let mut canvas = Canvas::new(DRAW_X as usize, DRAW_Y as usize, FUNC_X, FUNC_Y, lattice_size as usize);

    //let eq = |x:f64, y:f64| x.sin() - y;
    // let eq = |x:f64, y:f64| (1.0/x).sin() - y;
    //let eq = |x:f64, y:f64| x*x + y*y - 3.0;
    // let eq = |x:f64, y:f64| x.sin() - y.cos() + 2.0*x.sin()*y.sin();
    //let eq = |x:f64, y:f64| x.sin() - y.cos() - 2.0*x.sin()/y.sin();
    // let eq = |x:f64, y:f64| x.sin() - y.cos() - 2.0*x.sin()/y.sin() + (2.0*x).cos();
    // let eq = |x:f64, y:f64| (x*x).sin() - (y*y).cos();
    // let eq = |x:f64, y:f64| (x/y).sin();
    //let eq = |x:f64, y:f64| (1.0/x+1.0/y).sin()+x.sin();
    //let eq = |x:f64, y:f64| x.sin()-y.cos()-2.0*(x.sin())/(y.sin())+(2.0cos(2x)
    // let eq = |x:f64, y:f64| x.sin()- 3.0*x.cos()*(y/100.0).sin() + y.sin() - (3.0*y).cos()*(x/100.0).sin();
    // let eq = |x:f64, y:f64| - x/y;
    // turtle.drawing_mut().set_size((DRAW_X, DRAW_Y));
    // turtle.hide();
    // turtle.set_pen_size(1.0);
    // turtle.set_speed("instant");

// fn view(app: &App, frame: Frame) -> Frame  {
//     let canvas = unsafe{ c_pointer.as_ref().unwrap()};
//     // let draw = app.draw();
//     // turtle.drawing_mut().set_title(&format!("lattice {}x{}",lattice_size, lattice_size));
//     // for i in 0..DRAW_X{
//     //     for j in 0..DRAW_Y{
//     //         // if canvas.is_color_match(i as usize, j as usize, 0){
//     //              draw.rect()
//     //                 .x_y(i as f32, j as f32)
//     //                 .w(1.0)
//     //                 .h(1.0)
//     //                 .color(LIGHT_YELLOW);
//     //             // turtle.point((i as i32 - x_offset) as f64, (j as i32 - y_offset) as f64);
//     //         // }
//     //     }
//     // }
//     // draw.to_frame(app, &frame).unwrap();
//     frame.blit_from_simple_framebuffer(canvas);
//     println!("frame");
//     frame
// }

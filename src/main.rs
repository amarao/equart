extern crate minifb;

use itertools::Itertools;
use std::time::{Duration, Instant};
use std::thread::sleep;
use minifb::{Key, WindowOptions, Window};
fn sin(x:f64) ->f64 {
    x.sin()
}
fn cos(x:f64) -> f64 {
    x.cos()
}

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
    value: &'a mut u32,
    cartesian: (f64,f64), //Cartesian coordinates
    size: (f64, f64),
    lattice_dim: usize
}
impl<'a> Pixel<'a> {

    fn iterate_lattice(&self) -> impl Iterator<Item = (f64, f64)>{
        let x = self.cartesian.0;
        let y = self.cartesian.1;
        let dx = self.size.0;
        let dy = self.size.1;
        let lattice = self.lattice_dim;
        let conv = move |(i, j): (usize, usize)| (x+dx/(lattice - 1) as f64 * i as f64, y+dy/(lattice - 1) as f64 * j as f64);
        let new_it = (0..self.lattice_dim).cartesian_product(0..self.lattice_dim).map(conv);
        new_it
    }

    fn sign_change_on_lattice<F> (&self, func:F) -> bool where
        F: Fn(f64, f64) -> f64
    {
        let mut sign: Option<bool> = None;
        for (x,y) in self.iterate_lattice(){
            let res = func(x,y);
            if !res.is_finite() {continue};
            let num_sign = res.signum() > 0.0;
            sign = match sign {
                None => {Some(num_sign)},
                Some(old_sign) if old_sign != num_sign => { return true },
                _ => {continue}
            };
        }
        false
    }
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
        let img_size = ((canvas_x as u64)*(canvas_y as u64)) as usize;
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
    fn iter_mut<'iter>(&'iter mut self) ->  impl Iterator<Item = Pixel<'iter>>{
        let pixel_x = self.pixel_x;
        let pixel_size_x = self.pixel_size_x;
        let pixel_size_y = self.pixel_size_y;
        let zero_x = self.zero_x;
        let zero_y = self.zero_y;
        let lattice_dim = self.lattice_dim;
        println!("Start: {}, {}, {}", zero_x, zero_y, pixel_x);
        let calc = move |i| {
            // println!("pixel #{}", i);
            let row = i as i64 / pixel_x as i64 - zero_y as i64;
            // println!("Pixel row: {}, data: {}/{} - {}", row,i, pixel_x, zero_y);
            let column = i as i64 %  pixel_x  as i64 - zero_x as i64;
            // println!("Pixel row: {}, column: {}", row,column);
            let y = (row as f64) * pixel_size_y;
            let x = (column as f64) * pixel_size_y;
            // println!("Pixel at {}, {}", x,y);
            (x,y)
        };
        (&mut self.img).iter_mut().enumerate().map(
             move |(i, value)| Pixel{value, cartesian: calc(i), size: (pixel_size_x, pixel_size_y), lattice_dim}
        //    move |(i, value)| Pixel{value, cartesian: (0.0, 0.0), size: (pixel_size_x, pixel_size_y), lattice_dim}
        )
    }

}

fn show_and_wait(canvas:Canvas){
    let mut window = Window::new("Test - ESC to exit",
                                 canvas.pixel_x,
                                 canvas.pixel_y,
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


fn main() {
    // let picture = (
    //     |x:f64, y:f64| x.sin()-y,
    //     1.92*2.0,
    //     1.08*2.0,
    // );
    // let picture = (
    //     |x:f64 ,y:f64| (x*x).sin() - (y*y).cos(),
    //     1.92*20.0,
    //     1.08*20.0,
    //     "circles"
    // );
    let picture = (
        |x:f64, y:f64| sin(x)/sin(y)-cos(x*y),
        1.92*64.0,
        1.08*64.0,
        "wiggle-squares"
    );
    let mut canvas = Canvas::new(
        1920,1080,
        picture.1, picture.2,
        1920/2, 1080/2, 9,
    );
    for pixel in canvas.iter_mut(){
        if pixel.sign_change_on_lattice(picture.0){
            *pixel.value = 0;
        }
    }
    show_and_wait(canvas);
}
//     let eq = |x:f64, y:f64| (x*x).sin() - (y*y).cos();
//     // let x_offset = (DRAW_X/2) as i32;
//     // let y_offset = (DRAW_Y/2) as i32;
//     for pixel in canvas.into_iter(){
//         let sign_change = sign_change_on_lattice(&pixel, eq, lattice_size);
//             //turtle.point(i as f64, j as f64);
//         //canvas.dot(&pixel, eq(pixel.x, pixel.y));
//     }
//     //canvas.finish();
//     //show_and_wait(canvas);
// }
//
//     //let mut turtle = Turtle::new();
//     //let mut canvas = Canvas::new(DRAW_X as usize, DRAW_Y as usize, FUNC_X, FUNC_Y, lattice_size as usize);
//
//     //let eq = |x:f64, y:f64| x.sin() - y;
//     // let eq = |x:f64, y:f64| (1.0/x).sin() - y;
//     //let eq = |x:f64, y:f64| x*x + y*y - 3.0;
//     // let eq = |x:f64, y:f64| x.sin() - y.cos() + 2.0*x.sin()*y.sin();
//     //let eq = |x:f64, y:f64| x.sin() - y.cos() - 2.0*x.sin()/y.sin();
//     // let eq = |x:f64, y:f64| x.sin() - y.cos() - 2.0*x.sin()/y.sin() + (2.0*x).cos();
//     // let eq = |x:f64, y:f64| (x*x).sin() - (y*y).cos();
//     // let eq = |x:f64, y:f64| (x/y).sin();
//     //let eq = |x:f64, y:f64| (1.0/x+1.0/y).sin()+x.sin();
//     //let eq = |x:f64, y:f64| x.sin()-y.cos()-2.0*(x.sin())/(y.sin())+(2.0cos(2x)
//     // let eq = |x:f64, y:f64| x.sin()- 3.0*x.cos()*(y/100.0).sin() + y.sin() - (3.0*y).cos()*(x/100.0).sin();
//     // let eq = |x:f64, y:f64| - x/y;
//     // turtle.drawing_mut().set_size((DRAW_X, DRAW_Y));
//     // turtle.hide();
//     // turtle.set_pen_size(1.0);
//     // turtle.set_speed("instant");
//
// // fn view(app: &App, frame: Frame) -> Frame  {
// //     let canvas = unsafe{ c_pointer.as_ref().unwrap()};
// //     // let draw = app.draw();
// //     // turtle.drawing_mut().set_title(&format!("lattice {}x{}",lattice_size, lattice_size));
// //     // for i in 0..DRAW_X{
// //     //     for j in 0..DRAW_Y{
// //     //         // if canvas.is_color_match(i as usize, j as usize, 0){
// //     //              draw.rect()
// //     //                 .x_y(i as f32, j as f32)
// //     //                 .w(1.0)
// //     //                 .h(1.0)
// //     //                 .color(LIGHT_YELLOW);
// //     //             // turtle.point((i as i32 - x_offset) as f64, (j as i32 - y_offset) as f64);
// //     //         // }
// //     //     }
// //     // }
// //     // draw.to_frame(app, &frame).unwrap();
// //     frame.blit_from_simple_framebuffer(canvas);
// //     println!("frame");
// //     frame
// // }

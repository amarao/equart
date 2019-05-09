extern crate minifb;

use itertools::Itertools;
use std::time::{Duration, Instant};
use std::thread::sleep;
use minifb::{Key, WindowOptions, Window};
use std::f32::consts::*;

type Float = f32;
fn sin(x:Float) ->Float {
    x.sin()
}
fn cos(x:Float) -> Float {
    x.cos()
}



// trait Point {
//     fn point(&mut self, x:Float, y:Float);
// }
//
// impl Point for Turtle {
//     fn point(&mut self, x:Float, y:Float){
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
    cartesian: (Float,Float), //Cartesian coordinates
    size: (Float, Float),
    lattice_dim: usize
}
impl<'a> Pixel<'a> {

    fn iterate_lattice(&self) -> impl Iterator<Item = (Float, Float)>{
        let x = self.cartesian.0;
        let y = self.cartesian.1;
        let dx = self.size.0;
        let dy = self.size.1;
        let lattice = self.lattice_dim;
        let conv = move |(i, j): (usize, usize)| (x+dx/(lattice - 1) as Float * i as Float, y+dy/(lattice - 1) as Float * j as Float);
        let new_it = (0..self.lattice_dim).cartesian_product(0..self.lattice_dim).map(conv);
        new_it
    }

    fn sign_change_on_lattice<F> (&self, func:F) -> bool where
        F: Fn(Float, Float) -> Float
    {
        let mut sign: Option<bool> = None;
        for (x,y) in self.iterate_lattice(){
            let res = func(x,y);
            if !res.is_finite() {return false};
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
    cartesian_x: Float,
    cartesian_y: Float,
    pixel_size_x: Float,
    pixel_size_y: Float,
    zero_x: usize,
    zero_y: usize,
    lattice_dim: usize,
}

impl Canvas{
    fn new(canvas_x: usize, canvas_y:usize, cartesian_x: Float, cartesian_y:Float, zero_position_x: usize, zero_position_y: usize, lattice_dim: usize) -> Canvas {
        let img_size = ((canvas_x as u64)*(canvas_y as u64)) as usize;
        let canvas = Canvas{
                img: vec![0xFFFF_FFFF;img_size],
                pixel_x: canvas_x,
                pixel_y: canvas_y,
                zero_x: zero_position_x,
                zero_y: zero_position_y,
                cartesian_x: cartesian_x,
                cartesian_y: cartesian_y,
                pixel_size_x: cartesian_x/(canvas_x as Float),
                pixel_size_y: cartesian_y/(canvas_y as Float),
                lattice_dim:lattice_dim,
        };
        canvas
    }
    fn iter_mut<'iter>(&'iter mut self) ->  impl Iterator<Item = Pixel<'iter>>{
        let pixel_x = self.pixel_x;
        let pixel_size_x = self.pixel_size_x;
        let pixel_size_y = self.pixel_size_y;
        println!("pixel size: {}x{}", pixel_size_x, pixel_size_y);
        let zero_x = self.zero_x;
        let zero_y = self.zero_y;
        let lattice_dim = self.lattice_dim;
        let calc = move |i| {
            let row = i as i64 / pixel_x as i64 - zero_y as i64;
            let column = i as i64 %  pixel_x  as i64 - zero_x as i64;
            let y = (row as Float) * pixel_size_y;
            let x = (column as Float) * pixel_size_x;
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
    //     |x:Float, y:Float| x.sin()-y,
    //     1.92*2.0,
    //     1.08*2.0,
    // );
    // let picture = (
    //     |x:Float ,y:Float| (x*x).sin() - (y*y).cos(),
    //     1.92*20.0,
    //     1.08*20.0,
    //     "circles"
    // );
    // let picture = (
    //     |x:Float, y:Float| sin(x)/sin(y)-cos(x*y),
    //     1.92*64.0,
    //     1.08*64.0,
    //     "wiggle-squares"
    // );
    // let picture = (|x:Float, y:Float| sin(1.0/x)-sin(1.0/y), 1.92*5.0, 1.08/5.0, "curve in cross");
    // let picture = (|x:Float, y:Float| sin(x)-cos(y)-sin(x/cos(y)), 1.92*100.0, 1.08*11.8, "beads");
    // let picture = (|x:Float, y:Float| sin(x*x/y)-cos(y*y/x), 1.92*100.0, 1.08*100.0, "butterfly");
    // let picture = (|x:Float, y:Float| x-y, 300.0, 3.0, "butterfly");
    // let picture = (|x:Float, y:Float| (x*x+y*y)*sin(x*y)-PI, 1.92*42.0, 1.08*42.0, "muare");
    let picture = (|x:Float, y:Float| (x*x+y*y)*sin(x*y)-PI, 1.92*100.0, 1.08*100.0, "darkness come");
    let picture = (|x:Float, y:Float| (x*x+y*y)*sin(x*y)-PI, 1.92*470.0, 1.08*470.0, "sea of solicitude");
    let mut canvas = Canvas::new(
        1920,1080,
        picture.1, picture.2,
        1920/2, 1080/2, 17,
    );
    let now = Instant::now();
    for pixel in canvas.iter_mut(){
        if pixel.sign_change_on_lattice(picture.0){
            *pixel.value = 0;
        }
    }
    println!("Rendered in {:#?}", now.elapsed());
    show_and_wait(canvas);
}

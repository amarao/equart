extern crate minifb;
extern crate clipboard;
extern crate lodepng;

use itertools::Itertools;
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::cmp;
use minifb::{Key, WindowOptions, Window};

use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use lodepng::encode32 as png_encode;


type Float = f32;
fn sin(x:Float) ->Float {
    x.sin()
}
fn cos(x:Float) -> Float {
    x.cos()
}

#[derive (PartialEq)]
struct Pixel {
    index: usize,
}

impl  Pixel {
    fn as_pixel(&self, canvas: &Canvas) -> [i64; 2]{
        let row = self.index as i64 / canvas.pixel_x as i64 - canvas.zero_y as i64;
        let column = self.index as i64 %  canvas.pixel_x  as i64 - canvas.zero_x as i64;
        [row, column]
    }
    fn as_cartesian(&self, canvas: &Canvas) -> [Float; 2]{
        let [row, column] = self.as_pixel(canvas);
        let y = (row as Float) * canvas.pixel_size_y;
        let x = (column as Float) * canvas.pixel_size_x;
        [x,  y]
    }
    fn iterate_lattice_as_cartesian(&self, canvas: &Canvas, lattice_dim:usize) -> impl Iterator<Item =[Float;2]> {
        let [x,y] = self.as_cartesian(canvas);
        let (dx, dy) = (canvas.pixel_size_x, canvas.pixel_size_y);
        let subcanvas = (lattice_dim - 1) as Float;
        let conv = move |(i, j): (usize, usize)| {
                [
                    x+dx/subcanvas * i as Float,
                    y+dy/subcanvas * j as Float
                ]
            };
        (0..canvas.lattice_dim).cartesian_product(0..canvas.lattice_dim).map(conv)
    }

    fn sign_change_on_lattice<F> (&self, func:F, canvas: &Canvas, lattice_dim:usize) -> bool where
        F: Fn(Float, Float) -> Float
    {
        let mut sign: Option<bool> = None;
        for [x, y] in self.iterate_lattice_as_cartesian(canvas, lattice_dim){
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
    fn iter(&self) ->  impl Iterator<Item = Pixel>{
        (0..(self.pixel_x*self.pixel_y)).into_iter().map(|x|Pixel{index: x as usize})
    }

    fn get_neighbors(& self, pixel: & Pixel) -> Vec<Pixel>{
        let mut res: Vec<Pixel> = Vec::with_capacity(8);
        for x in -1..2{
            for y in -1..2 {
                if x==y && y==0 { continue };
                let neighbor = pixel.index as i64 + y as i64 *self.pixel_x as i64 + x as i64;
                if neighbor < 0 || neighbor >= self.pixel_x as i64 *self.pixel_y as i64{
                    continue;
                }
                res.push(Pixel{index: neighbor as usize});
            }
        }
        res
    }
    fn set_pixel(& mut self, pixel: &Pixel, value: u32) {
        self.img[pixel.index] = value;
    }
    fn get_pixel(&self, pixel:&Pixel) -> u32 {
        self.img[pixel.index]
    }
}

fn copy_to_clipboard(canvas: &Canvas){
    // let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    // ctx.set_contents(&canvas.img);
    println!("encoding start");
    let png = png_encode(&canvas.img, canvas.pixel_x, canvas.pixel_y).unwrap();
    // use std::mem::size_of_val;
    println!("encoding end, size: {}", png.len());
    println!("Copying to clipboard");
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
        if window.is_key_down(Key::Enter){
            copy_to_clipboard(&canvas)
        }
        let start = Instant::now();
        window.update();
        let spend = start.elapsed();
        sleep(Duration::new(0,1000000000/60).checked_sub(spend).unwrap_or(Duration::new(0,0)));
    }
}

fn render<F>(canvas: &mut Canvas, f: &F) where
    F: Fn(Float, Float) -> Float
{
    for pixel in canvas.iter(){
        if pixel.sign_change_on_lattice(f, &canvas, canvas.lattice_dim){
            canvas.set_pixel(&pixel, 0);
        }
    }
}

fn clarify<F>(canvas: &mut Canvas, f: &F) -> u64 where
    F: Fn(Float, Float) -> Float
{
    let mut update_count = -1;
    let mut iteration = 0;

    let mut last_roots: Vec<Pixel> = Vec::new();
    for _ in 0..6 {
        while update_count !=0  {
            update_count = 0;
            iteration += 1;
            let mut new_roots: Vec<Pixel> = Vec::new();
            let mut max_impact = 1;
            for pixel in canvas.iter(){
                if canvas.get_pixel(&pixel) != 0 {
                    let mut impact = iteration;
                    for neighbor in canvas.get_neighbors(&pixel).iter() {
                        if canvas.get_pixel(neighbor) == 0u32 {
                                if last_roots.contains(neighbor) {
                                    impact *= 10;
                                }
                                impact *= 2;
                        }
                    }
                    if impact != 1 {
                        if pixel.sign_change_on_lattice(f, &canvas, (canvas.lattice_dim as u64 * impact) as usize){
                            max_impact = cmp::max(impact, max_impact);
                            canvas.set_pixel(&pixel, 0);
                            update_count += 1;
                            new_roots.push(pixel);
                        }
                    }
                }
            }
            last_roots = new_roots;
            println!("Updating: iteration {}  max lattice:{}, {} additional pixels", iteration, iteration * max_impact,  update_count);
        }
        update_count=-1;
    }
    iteration
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
    let picture = (|x:Float, y:Float| sin(1.0/x)-sin(1.0/y), 1.92*5.0, 1.08/5.0, "curve in cross");
    // let picture = (|x:Float, y:Float| sin(x)-cos(y)-sin(x/cos(y)), 1.92*100.0, 1.08*11.8, "beads");
    // let picture = (|x:Float, y:Float| sin(x*x/y)-cos(y*y/x), 1.92*100.0, 1.08*100.0, "butterfly");
    // let picture = (|x:Float, y:Float| x-y, 300.0, 3.0, "butterfly");

    // let picture = (|x:Float, y:Float| sin(x/y)-sin(y/x), 1.92*100.0, 1.08/100.0, "?");
    // let picture = (|x:Float, y:Float| (sin(x)+sin(y/2.0))*(sin(x)+sin(x/2.0)-y), 1.92*20.0, 1.08*20.0, "two quarters");
    let mut canvas = Canvas::new(
        1920,1080,
        picture.1, picture.2,
        1920/2, 1080/2, 2,
    );
    let now = Instant::now();
    render(&mut canvas, &picture.0);
    println!("Rendered in {:#?}", now.elapsed());
    clarify(&mut canvas, &picture.0);
    println!("Finish rendering and updates in {:#?}", now.elapsed());
    show_and_wait(canvas);
}

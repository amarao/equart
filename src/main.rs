#![allow(dead_code)]

use std::time::Instant;
use std::cmp;
use std::cmp::max as max;

// use lodepng::encode32 as png_encode;
use equart::pix::*;

extern crate piston_window;
extern crate image as im;

use piston_window::*;
use piston::event_loop::Events;
// use vecmath::*;

// fn copy_to_clipboard(img: &Vec<u32>, x:usize, y:usize){
//     let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
//     println!("encoding start");
//     let png = png_encode(img, x, y).unwrap();
//     println!("encoding end, size: {}", png.len());
//     println!("Copying to clipboard");
//     // ctx.set_contents(png);
// }


fn draw_and_calc(){
    let X: u32 = 1920;
    let Y: u32 = 1080;

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("equart", (X, Y))
        .exit_on_esc(true)
        .graphics_api(opengl)
        .build()
        .unwrap();

    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };



    let picture = (|x:Float, y:Float| sin(x)*x/2.0 + x - y, X as f32, Y as f32, "inverse test");

    let mut cnv = Canvas::new(
        X as usize, Y as usize,
        picture.1, picture.2,
        1920/2, 1080/2
    );
    let mut stage = 1;
    let mut old_roots = 0;
    let mut new_roots = 0;
    let mut events = Events::new(EventSettings::new().lazy(false));
    let mut lattice_dim = 2;
    while let Some(e) = events.next(&mut window) {
        if let Some(_) = e.render_args() {
            let now = Instant::now();
            match stage {
                0 => { },
                _ => {
                    println!("Stage {}", stage);
                    println!("Bruteforcing with lattice {}x{}", lattice_dim, lattice_dim);
                    render(&mut cnv, &picture.0, 2, max(0, 200 - lattice_dim));
                    new_roots = cnv.roots();
                    let found_roots = new_roots - old_roots;
                    if found_roots  > 0 {
                        println!("Found {} new roots.", found_roots);
                        lattice_dim = lattice_dim * 2 + 1;
                        old_roots = new_roots;
                        stage = stage + 1;
                    }
                    else {
                        events.set_lazy(true);
                        println!("Done rendering, lazy mode activated");
                        stage = 0;
                    }
                }
            }
            let canvas = im::ImageBuffer::from_fn(X, Y, |x, y| {
                    let v = cnv.img[(x + y * cnv.pixel_x as u32) as usize];
                    im::Rgba([v,v,v, 255])
            });
            let mut texture: G2dTexture = Texture::from_image(
                    &mut texture_context,
                    &canvas,
                    &TextureSettings::new()
                ).unwrap();
            texture.update(&mut texture_context, &canvas).unwrap();
            window.draw_2d(&e, |c, g, _device| {
                // Update texture before rendering.
                // texture_context.encoder.flush(_device);

                // clear([1.0; 4], g);
                image(&texture, c.transform, g);
            });
        }
    }

}



fn render<F>(canvas: &mut Canvas, f: &F, lattice_dim:usize, color:PixelColor) where
    F: Fn(Float, Float) -> Float
{
    for pixel in canvas.iter(){
        if canvas.img[pixel.index]!=0 as PixelColor {
            if pixel.sign_change_on_lattice(f, &canvas, lattice_dim){
                canvas.set_pixel(&pixel, color);
            }
        }
    }
}


// fn clarify<F>(canvas: &mut Canvas, f: &F, lattice_dim:usize) -> u64 where
//     F: Fn(Float, Float) -> Float
// {
//     let mut update_count = -1;
//     let mut iteration = 0;
//     let mut scandepth: Vec<u16> = vec![lattice_dim as u16;canvas.img.len()];
//     for attempt in 1..4 {
//         let mut last_roots: Vec<Pixel> = Vec::new();
//         while update_count !=0  {
//             update_count = 0;
//             iteration += 1;
//             let mut max_boost = 1;
//             let scan_lattice = (lattice_dim as u64 + iteration)  as usize * (attempt);
//             let deepscan_lattice = (lattice_dim as u64 + iteration * 5) as usize * (attempt);
//             let mut pix_count = 0;
//             for pixel in canvas.iter(){
//                 if canvas.get_pixel(&pixel) != 0 {
//                     if scandepth[pixel.index] < scan_lattice as u16{
//                         scandepth[pixel.index] = scan_lattice as u16;
//                         if canvas.neighbors_roots_count(&pixel) > 0 {
//                             pix_count += 1;
//                             if pixel.sign_change_on_lattice(f, &canvas, scan_lattice){
//                                 max_boost = cmp::max(iteration, max_boost);
//                                 canvas.set_pixel(&pixel, 0);
//                                 update_count += 1;
//                                 last_roots.push(pixel);
//                             }
//                         }
//                     }
//                 }
//             }
//             println!("Scanned {} neighbors of all old roots, found {} new roots, lattice {}", pix_count, update_count, scan_lattice);
//
//             while last_roots.len() !=0 {
//                 let mut pix_count = 0;
//                 let mut new_roots: Vec<Pixel> = Vec::new();
//                 for last_root in last_roots.iter(){
//                     for neighbor in canvas.get_neighbors(&last_root){
//                         if scandepth[neighbor.index] < deepscan_lattice as u16{
//                             scandepth[neighbor.index] = deepscan_lattice as u16;
//                             pix_count += 1;
//                             if canvas.get_pixel(&neighbor) != 0 as PixelColor{
//                                 if neighbor.sign_change_on_lattice(f, &canvas, deepscan_lattice){
//                                     max_boost = cmp::max(iteration, max_boost);
//                                     canvas.set_pixel(&neighbor, 0);
//                                     update_count += 1;
//                                     new_roots.push(neighbor);
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 println!("Deep scanned {} pixels (neighbors of {} new roots), found {} more new neighbor roots at lattice {}", pix_count, last_roots.len(), new_roots.len(), deepscan_lattice);
//                 last_roots = new_roots.clone();
//             }
//             println!("Finished iteration {}, found {} additional pixels", iteration, update_count);
//         }
//         update_count=-1;
//     }
//     iteration
// }

fn main() {
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
    // let picture = (|x:Float, y:Float| sin(1.0/x)-y, 1.92/100.0, 1.08*2.0, "test");
    // let picture = (|x:Float, y:Float| sin(1.0/x)-sin(1.0/y), 1.92*5.0, 1.08/5.0, "curve in cross");
    // let picture = (|x:Float, y:Float| sin(x)-cos(y)-sin(x/cos(y)), 1.92*100.0, 1.08*11.8, "beads");
    // let picture = (|x:Float, y:Float| sin(x*x/y)-cos(y*y/x), 1.92*100.0, 1.08*100.0, "butterfly");
    // let picture = (|x:Float, y:Float| x-y, 300.0, 3.0, "butterfly");
    // let picture = (|x:Float, y:Float| sin(x/y)-sin(y/x), 1.92*100.0, 1.08/100.0, "?");
    // let picture = (|x:Float, y:Float| (sin(x)+sin(y/2.0))*(sin(x)+sin(x/2.0)-y), 1.92*20.0, 1.08*20.0, "two quarters");
    // let picture = (|x:Float, y:Float| (x*x+y*y)*sin(x*y)-PI, 1.92*42.0, 1.08*42.0, "muare");
    // let picture = (|x:Float, y:Float| (x*x+y*y)*sin(x*y)-PI, 1.92*100.0, 1.08*100.0, "darkness come");
    // let picture = (|x:Float, y:Float| (x*x+y*y)*sin(x*y)-PI, 1.92*470.0, 1.08*470.0, "sea of solicitude");
    // let picture = (|x:Float, y:Float| sin(x*cos(y))-cos(y*sin(x)), 1.92*60.0, 1.08*60.0, "tarnished lace");
    // let picture = (|x:Float, y:Float| sin(x/y)-cos(y/x)+x-y, 1.92*2.8, 1.08*2.8, "trimed knot");
    // let picture = (|x:Float, y:Float| (x+sin(1.0/x)-1.0/sin(x))/10.0-(y/x).exp().sin(), 1.92*3.0, 1.08*12.0, "?");
    // let picture = (|x:Float, y:Float| x/sin(y)-y*y*cos(x), 1.92*64.0, 1.08*64.0, "square soap");
    // let picture = (|x:Float, y:Float| sin(x+2.0*y)-cos(x*y*y), 1.92*32.0, 1.08*8.0, "?");
    // let picture = (|x:Float, y:Float| (y*y+x*x-(1.0-0.2*(x/y).atan()).sin()), 1.92*4.0, 1.08*4.0, "?");
    // let picture = (|x:Float, y:Float| (sin(y)+sin(x+y)+sin(x)-((x/y).atan()).sin()), 1.92*200.0, 1.08*200.0, "!");
    // let picture = (|x:Float, y:Float| (sin(y)+cos(x+y)+sin(x)-((x/y).atan()).cos()), 1.92*256.0, 1.08*256.0, "circle doodling");
    draw_and_calc();
}

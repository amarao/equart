extern crate turtle;
extern crate imgref;

use turtle::Turtle;

trait Point {
    fn point(&mut self, x:f64, y:f64);
}

impl Point for Turtle {
    fn point(&mut self, x:f64, y:f64){

        self.pen_up();
        self.go_to([x, y]);
        self.pen_down();
        self.forward(1.0);
        self.pen_up();
    }
}


struct Canvas {
    canvas: Vec<u8>,
    pix_dim_x: usize,
    pix_dim_y: usize,
    real_size_x: f64,
    real_size_y: f64,
    //real_min_x: f64, //offset
    //real_min_y: f64,
    lattice_dim: usize,
}

impl Canvas{
    fn new(pix_dim_x: usize, pix_dim_y: usize, real_size_x: f64, real_size_y: f64, lattice_dim: usize) -> Canvas {
        let canv = Vec::with_capacity(pix_dim_x * pix_dim_y);
        Canvas{canvas: canv, pix_dim_x: pix_dim_x, pix_dim_y: pix_dim_y, real_size_x: real_size_x, real_size_y: real_size_y, lattice_dim:lattice_dim}
    }
    // fn dot(&self, i:usize, j:usize){
    //
    // }
}



const DRAW_X: u32 = 1920;
const DRAW_Y: u32 = 1080;
const ZOOM: f64 = 20.0;
const FUNC_X: f64 = 1.92*ZOOM;
const FUNC_Y: f64 = 1.08*ZOOM/64.0;


struct Pixel{x:f64, y:f64, dx:f64, dy:f64}

fn conv(i: i32, j:i32) -> Pixel {
    let x = (i as f64)/(DRAW_X as f64) * FUNC_X;
    let y = (j as f64)/(DRAW_Y as f64) * FUNC_Y;
    let dx = FUNC_X/(DRAW_X as f64);
    let dy = FUNC_Y/(DRAW_Y as f64);
    Pixel{x, y, dx, dy}
}

fn sign_change_on_lattice<F> (pixel:Pixel, func: F, dim: u8) -> bool where
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

fn main() {
    let mut turtle = Turtle::new();
    //let eq = |x:f64, y:f64| x.sin() - y;
    // let eq = |x:f64, y:f64| (1.0/x).sin() - y;
    //let eq = |x:f64, y:f64| x*x + y*y - 3.0;
    // let eq = |x:f64, y:f64| x.sin() - y.cos() + 2.0*x.sin()*y.sin();
    // let eq = |x:f64, y:f64| x.sin() - y.cos() - 2.0*x.sin()/y.sin();
    // let eq = |x:f64, y:f64| (x*x).sin() - (y*y).cos();
    // let eq = |x:f64, y:f64| (x/y).sin();
    let eq = |x:f64, y:f64| (1.0/x+1.0/y).sin()+x.sin();
    turtle.drawing_mut().set_size((DRAW_X, DRAW_Y));
    turtle.hide();
    turtle.set_pen_size(1.0);
    turtle.set_speed("instant");
    for lattice_size in &[63u8]{
        turtle.drawing_mut().set_title(&format!("lattice {}x{}",&lattice_size, &lattice_size));
        for i in -(DRAW_X as i32)/2..(DRAW_X as i32)/2 {
            for j in -(DRAW_Y as i32)/2..(DRAW_Y as i32)/2 {
                let pixel = conv(i, j);
                if sign_change_on_lattice(pixel, eq, *lattice_size){
                    turtle.point(i as f64, j as f64);
                }
            }
        }
    }
}

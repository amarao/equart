extern crate turtle;

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

const DRAW_X: u32 = 1024;
const FUNC_X: f64 = 4.0;
const DRAW_Y: u32 = 1024;
const FUNC_Y: f64 = 4.0;


struct Pixel{x:f64, y:f64, dx:f64, dy:f64}

fn conv(i: i32, j:i32) -> Pixel {
    let x = (i as f64)/(DRAW_X as f64) * FUNC_X;
    let y = (j as f64)/(DRAW_Y as f64) * FUNC_Y;
    let dx = FUNC_X/(DRAW_X as f64);
    let dy = FUNC_Y/(DRAW_Y as f64);
    Pixel{x, y, dx, dy}
}


fn within<F>(pixel:Pixel, eq: F) -> bool where
    F: Fn(f64, f64) -> f64
{
    let v1 = eq(pixel.x + pixel.dx/2.0, pixel.y + pixel.dy/2.0);
    if v1.abs() < ((pixel.dx/2.0).abs()).min((pixel.dy/2.0).abs()){
        return true
    }
    // let v2 = eq(pixel.x+pixel.dx, pixel.y);
    // let v3 = eq(pixel.x, pixel.y+pixel.dy);
    // let v4 = eq(pixel.x + pixel.dx, pixel.y + pixel.dy);
    // if [v1,v2,v3,v4].iter().map(abs).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() < &((pixel.dx).min(pixel.dy)) {
    //     return true
    // }
    false
}

fn main() {
    let mut turtle = Turtle::new();
    let eq = |x:f64, y:f64| x.sin() - y;
    turtle.drawing_mut().set_size((DRAW_X+50, DRAW_Y+50));
    turtle.set_speed("instant");
    for i in -(DRAW_X as i32)/2..(DRAW_X as i32)/2 {
        turtle.right(360.0/(DRAW_X as f64));
        for j in -(DRAW_Y as i32)/2..(DRAW_Y as i32)/2 {
            let pixel = conv(i, j);
            if within(pixel, eq){
                turtle.point(i as f64, j as f64);
            }
        }
    }
}

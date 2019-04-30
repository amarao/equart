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

const DRAW_X: u32 = 1920;
const DRAW_Y: u32 = 1080;

const FUNC_X: f64 = 20.0;
const FUNC_Y: f64 = 20.0;


struct Pixel{x:f64, y:f64, dx:f64, dy:f64}

fn conv(i: i32, j:i32) -> Pixel {
    let x = (i as f64)/(DRAW_X as f64) * FUNC_X;
    let y = (j as f64)/(DRAW_Y as f64) * FUNC_Y;
    let dx = FUNC_X/(DRAW_X as f64);
    let dy = FUNC_Y/(DRAW_Y as f64);
    Pixel{x, y, dx, dy}
}

fn float_minmax<'a, T>(seq: T) -> (f64, f64) where
    T: Iterator<Item=&'a f64> + Clone
{
    let mut min: Option<f64> = None;
    let mut max: Option<f64> = None;
    for &num in seq {
        min = match min {
            Some(val) if val > num => {Some(num)},
            Some(val) => {Some(val)},
            None => {Some(num)},
        };
        max = match max {
            Some(val) if val < num => {Some(num)},
            Some(val) => {Some(val)},
            None => {Some(num)},
        }
    }
    //let max:f64 = seq.max_by(|a:&f64, b:&f64| (*a).partial_cmp(b).unwrap()).unwrap();
    // println!("{}, {}", min, max);
    (min.unwrap(), max.unwrap())
}

fn sign_change<'a, T>(points: T) -> bool where
    T: Iterator<Item=&'a f64> + Clone
{
    let (min_val, max_val) = float_minmax(points);
    // println!("{}, {}", min_val.signum(), max_val.signum());
    min_val.signum() != max_val.signum()
}

fn within<F>(pixel:Pixel, eq: F) -> bool where
    F: Fn(f64, f64) -> f64
{
    let x = pixel.x;
    let y = pixel.y;
    let dx = pixel.dx;
    let dy = pixel.dy;
    let v1 = eq(x + dx/2.0, y + dy/2.0);
    if v1.abs() < ((dx/2.0).abs()).min((dy/2.0).abs()){
        return true;
    }
    let corners: [f64;4] = [eq(x, y), eq(x+dx, y), eq(x, y+dy), eq(x+dx, y+dy)];

    sign_change(corners.iter())
}

fn main() {
    let mut turtle = Turtle::new();
    // let eq = |x:f64, y:f64| x.sin() - y;
    let eq = |x:f64, y:f64| (1.0/x).sin() - y;
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
#[cfg(test)]
mod tests{
    use super::float_minmax;
    #[test]
    fn test_float_minmax_2(){
        let data = [2.0, 2.0, 2.0, 2.0];
        assert_eq!(float_minmax(data.iter()), (2.0, 2.0))
    }
    #[test]
    fn test_float_minmax_diff(){
        let data = [-2.0, 2.0, 4.0, 2.0];
        assert_eq!(float_minmax(data.iter()), (-2.0, 4.0))
    }
}

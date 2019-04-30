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

const FUNC_X: f64 = 0.1;
const FUNC_Y: f64 = 3.0;


struct Pixel{x:f64, y:f64, dx:f64, dy:f64}

fn conv(i: i32, j:i32) -> Pixel {
    let x = (i as f64)/(DRAW_X as f64) * FUNC_X;
    let y = (j as f64)/(DRAW_Y as f64) * FUNC_Y;
    let dx = FUNC_X/(DRAW_X as f64);
    let dy = FUNC_Y/(DRAW_Y as f64);
    Pixel{x, y, dx, dy}
}

// fn float_minmax<'a, T>(seq: T) -> (f64, f64) where
//     T: Iterator<Item=&'a f64> + Clone
// {
//     let mut min: Option<f64> = None;
//     let mut max: Option<f64> = None;
//     for &num in seq {
//         min = match min {
//             Some(val) if val > num => {Some(num)},
//             Some(val) => {Some(val)},
//             None => {Some(num)},
//         };
//         max = match max {
//             Some(val) if val < num => {Some(num)},
//             Some(val) => {Some(val)},
//             None => {Some(num)},
//         }
//     }
//     //let max:f64 = seq.max_by(|a:&f64, b:&f64| (*a).partial_cmp(b).unwrap()).unwrap();
//     // println!("{}, {}", min, max);
//     (min.unwrap(), max.unwrap())
// }

fn sign_change<'a, T>(points: T) -> bool where
    T: Iterator<Item=&'a f64> + Clone
{
    let mut sign: Option<bool> = None;
    for &num in points {
        if num!=num {continue};
        let num_sign:bool = num.signum() > 0.0;
        sign = match sign {
            None => {Some(num_sign)},
            Some(old_sign) => {if old_sign != num_sign { return true } else {continue}}
        };
    }
    false
}

fn apply_on_lattice<F> (pixel:Pixel, func: F, dim: u8) -> Vec<f64> where
    F: Fn(f64, f64) -> f64
{
    let mut output = Vec::with_capacity((dim*dim) as usize);
    for i in 0..dim {
        for j in 0..dim{
            let x = pixel.x+pixel.dx/((dim - 1) as f64)*(i as f64);
            let y = pixel.y+pixel.dy/((dim - 1) as f64)*(j as f64);
            // println!("pos: {}, {}",x, y);
            output.push(func(x,y));
        }
    }
    // println!("values: {:?}",output);
    output
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
    // if x.abs() < 0.01{
        let points = apply_on_lattice(pixel, eq, 10);
        sign_change(points.iter())
    // }
    // else{
    //     let corners: [f64;4] = [eq(x, y), eq(x+dx, y), eq(x, y+dy), eq(x+dx, y+dy)];
    //     sign_change(corners.iter())
    // }
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
    use super::sign_change;
    #[test]
    fn test_sign_change(){
        let data = [2.0, 2.0, 2.0, 2.0];
        assert_eq!(sign_change(data.iter()), false);
    }
    #[test]
    fn test_float_minmax_diff(){
        let data = [-2.0, 2.0, 4.0, 2.0];
        assert_eq!(sign_change(data.iter()), true);
    }
}

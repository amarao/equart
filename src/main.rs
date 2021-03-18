use lib::EasyScreen;
mod quadtree;
pub use crate::quadtree::{QuadTree, Point};


fn equation(x: f64, y:f64) -> f64{
    ((((x.sin()-y).cos()-x).sin()-y).cos()/(x*y).sin()/(x.abs().ln()*(y*y).sin())).ln()
}

fn is_root(x: f64, y: f64, dx:f64, dy:f64) -> bool{
    let mut pos = 0;
    let mut neg = 0;
    if equation(x, y) > 0.0 {pos +=1} else {neg += 1};
    if equation(x+dx, y) > 0.0 {pos +=1} else {neg += 1};
    if equation(x, y+dy) > 0.0 {pos +=1} else {neg += 1};
    if equation(x+dx, y+dy) > 0.0 {pos +=1} else {neg += 1};
    (pos > 0) & (neg > 0)
}

fn draw(screen: &EasyScreen){
    let factor = 16.0;
    let x_start = 8.0;//-factor*2.56;
    let y_start = 8.0;//-factor*1.44;
    let x_end = 1.5*factor*2.56;
    let y_end = 1.5*factor*1.44;
    let mut q = QuadTree::from_coords(x_start, y_start, x_end, y_end);
    q.append_point(Point::new(9.0, 9.0), 0u32).unwrap();
    let dx = (x_end-x_start)/screen.width() as f64;
    let dy = (y_end-y_start)/screen.height() as f64;
    for y in 0..screen.height(){
        let real_y = y_start + dy*y as f64;
        for x in 0..screen.width(){
            let real_x = x_start + dx*x as f64;
            if is_root(real_x,real_y, dx, dy){
                screen.put_pixel(x, y, 0);
            }
        }
    }
    std::thread::park();
}

fn main(){
    let screen = EasyScreen::new();
    screen.fill(0xFFFFFFFF);
    draw(&screen);
    screen.wait();
}
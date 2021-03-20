use lib::EasyScreen;
mod quadtree;
pub use crate::quadtree::{QuadTree, Point};
use std::num::FpCategory;

#[derive (PartialEq)]
enum FixelState{
    NoProbes,
    NoDomain,
    NoRoot,
    ExactRoot,
    SignChangeRoot
}

struct Fixel{
    center: Point,
    dx: f64,
    dy: f64,
    probes: Vec<(Point, f64)>,
    state: FixelState,
    max: Option<f64>,
    min: Option<f64>,
}

impl Fixel{
    fn new(center: Point, dx: f64, dy: f64) -> Self{
        Fixel{
            center,
            dx,
            dy,
            probes: Vec::with_capacity(4),
            state: FixelState::NoProbes,
            max: None,
            min: None
        }
    }
    fn already_present(&self, new_point: Point) -> bool{
        for probe in self.probes.iter(){
            if probe.0 == new_point{
                return true
            }
        }
        false
    }

    fn update_cache(&mut self){
        if self.state == FixelState::ExactRoot || self.state == FixelState::NoDomain{
            return;
        }
        for probe in self.probes.iter(){
            match probe.1.classify(){
                FpCategory::Nan | FpCategory::Infinite => {
                    self.state = FixelState::NoDomain;
                    return;
                },
                FpCategory::Zero => {
                    self.state = FixelState::ExactRoot;
                    return;
                },
                _ => {}
            }
            self.max = Some(self.max.map_or(probe.1, |v| v.max(probe.1)));
            self.min = Some(self.min.map_or(probe.1, |v| v.min(probe.1)));
        }
        if self.max.is_some() && self.min.is_some() && self.max.unwrap() > 0.0 && self.min.unwrap() < 0.0 {
            self.state = FixelState::SignChangeRoot;
        }
    }
    fn gen_positions(&self, samples: usize) -> Vec<Point>{
        let mut positions = Vec::with_capacity(samples);
        let side = (samples as f64).sqrt().ceil() as usize;
        let x_step = self.dx/side as f64;
        let y_step = self.dy/side as f64;
        let start_x = self.center.x - self.dx/2.0;
        let start_y = self.center.y - self.dy/2.0;
        for x_cnt in 0..side{
            for y_cnt in 0..side{
                let point = Point::new(
                    start_x + x_step * x_cnt as f64,
                    start_y + y_step * y_cnt as f64,
                );
                positions.push(point);
            }
        }
        vec![Point::new(0.0, 0.0)]
    }

    /// Assure there is a least a 'samples' number of probes, if they are
    /// needed.
    fn do_probes<F>(&mut self, samples: usize, f: F)
    where F: Fn(Point) -> f64{
        if self.probes.len() >= samples || self.state == FixelState::ExactRoot || self.state == FixelState::SignChangeRoot{
            return 
        }
        for point in self.gen_positions(samples){
            if !self.already_present(point){
                self.probes.push((point, f(point)));
            }
        }
        self.update_cache();
    }
}

fn equation(x: f64, y:f64) -> f64{
    x.sin()*(y*3.0).sin()-x.sin()-y.sin()
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
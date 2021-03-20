use lib::EasyScreen;
mod quadtree;
pub use crate::quadtree::{QuadTree, Point};
use std::num::FpCategory;
use rand::Rng;

#[derive (PartialEq, Debug)]
enum FixelState{
    NoProbes,
    NoDomain,
    NoRoot,
    ExactRoot,
    SignChangeRoot
}

#[derive (Debug)]
struct Fixel{
    center: Point,
    dx: f64,
    dy: f64,
    screen_x: u32,
    screen_y: u32,
    probes: Vec<(Point, f64)>,
    state: FixelState,
    max: Option<f64>,
    min: Option<f64>,
}

impl Fixel{
    fn new(center: Point,screen_x: u32, screen_y: u32, dx: f64, dy: f64) -> Self{
        Fixel{
            center,
            dx,
            dy,
            screen_x,
            screen_y,
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

    fn update_cache(&mut self) -> bool{
        if self.state == FixelState::ExactRoot || self.state == FixelState::NoDomain{
            return false;
        }
        for probe in self.probes.iter(){
            match probe.1.classify(){
                FpCategory::Nan | FpCategory::Infinite => {
                    self.state = FixelState::NoDomain;
                    return true;
                },
                FpCategory::Zero => {
                    println!("Exact root!");
                    self.state = FixelState::ExactRoot;
                    return true;
                },
                _ => {}
            }
            self.max = Some(self.max.map_or(probe.1, |v| v.max(probe.1)));
            self.min = Some(self.min.map_or(probe.1, |v| v.min(probe.1)));
        }
        if self.max.is_some() && self.min.is_some() && self.max.unwrap() > 0.0 && self.min.unwrap() < 0.0 {
            self.state = FixelState::SignChangeRoot;
            return true
        }
        if self.state == FixelState::NoProbes{
            self.state = FixelState::NoRoot;
        }
        return false
    }
    fn gen_positions(&self, samples: usize, rng: &mut rand::rngs::ThreadRng) -> Vec<Point>{
        let mut positions = Vec::with_capacity(samples);
        for i in 0..samples{
                positions.push(
                    Point::new(
                        rng.gen_range(self.center.x - self.dx/2.0..=self.center.x + self.dx/2.0),
                        rng.gen_range(self.center.y - self.dy/2.0..=self.center.y + self.dy/2.0),
                    )
                );
        }
        positions
    }

    /// Assure there is a least a 'samples' number of probes, if they are
    /// needed.
    fn do_probes<F>(&mut self, samples: usize, f: F, rng: &mut rand::rngs::ThreadRng) -> bool
    where F: Fn(Point) -> f64{
        if self.probes.len() >= samples || self.state == FixelState::ExactRoot || self.state == FixelState::SignChangeRoot{
            return false
        }
        for point in self.gen_positions(samples, rng){
            if !self.already_present(point){
                self.probes.push((point, f(point)));
            }
        }
        self.update_cache()
    }

    fn color(&self) -> u32{
        match self.state {
            FixelState::ExactRoot| FixelState::SignChangeRoot => {
                0xFF000000
            },
            FixelState::NoProbes => {0x70707070},
            FixelState::NoRoot => {0xFFFFFFFF},
            FixelState::NoDomain => {0xBBBBBBBB}
        }
    }
}

#[derive (Debug)]
struct FixelArray{
    fixels: Vec<Fixel>,
    width: u32,
    height: u32,
    window_start: Point,
    window_end: Point,
}

impl FixelArray{
    fn new(width: u32, height:u32, window_start:Point, window_end: Point) -> Self{
        let dx = (window_end.x - window_start.x)/width as f64;
        let dy = (window_end.y - window_start.y)/height as f64;
        let mut fixels = Vec::with_capacity((width*height) as usize);
        for cnt_y in 0..height{
            for cnt_x in 0..width{
                fixels.push(
                    Fixel::new(
                        Point::new(
                            window_start.x + dx*cnt_x as f64 + dx/2.0,
                            window_start.y + dy*cnt_y as f64 + dy/2.0,
                        ),
                        cnt_x,
                        cnt_y,
                        dx,
                        dy
                    )
                );
            }
        }
        FixelArray{
            fixels,
            width,
            height,
            window_start,
            window_end
        }
    }

    fn do_probes<F>(&mut self, samples: usize, f: F, screen: &EasyScreen, rng: &mut rand::rngs::ThreadRng)
    where F: Fn(Point) -> f64{
        for fixel in self.fixels.iter_mut(){
            if fixel.do_probes(samples, &f, rng){
                screen.put_pixel(fixel.screen_x, fixel.screen_y, fixel.color());
            }
        }
    }
}

fn equation(p: Point) -> f64{
    // dbg!(p);
    p.x.sin() - p.y
}



fn draw(screen: &EasyScreen){
    let mut fixels = FixelArray::new(
        screen.width(),
        screen.height(),
        Point::new(-2.0, -2.0),
        Point::new(2.0, 2.0)
    );
    let mut rng = rand::thread_rng();
    let mut c = 3;
    loop{
        c+=2;
        println!("Doing probes with depth {}", c);
        fixels.do_probes(c, equation, screen, &mut rng);
    }
    std::thread::park();
}

fn main(){
    let screen = EasyScreen::new();
    screen.fill(0xFFFFFFFF);
    draw(&screen);
    screen.wait();
}
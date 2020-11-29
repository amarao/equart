use crate::fixel;
use crate::threads;
use threads::DrawingApp;
use image as im;
const WINDOW_X_START: f64 = -4.0;
const WINDOW_X_END: f64 = 4.0;
const WINDOW_Y_START: f64 = -1.5;
const WINDOW_Y_END: f64 = 1.5;

fn equart(x: f64, y:f64) -> f64{
    (1.0/x).sin() - y
}


pub struct Equart {  // per thread instance, each instance has own 'slice' to work with
    window_start: fixel::Point,
    window_end: fixel::Point,
    fixel_size_x: f64,
    fixel_size_y: f64,
    fixels: array2d::Array2D<fixel::Fixel>,
    max_target_depth: u32,
    min_achived_depth: u32,
    id: usize,
    max_id: usize,
}

impl DrawingApp for Equart{
    fn new(id: usize, max_id: usize, x: u32, y: u32)-> Self {
        let slice = Self::slice(WINDOW_Y_START, WINDOW_Y_END, id, max_id);
        Self{
            window_start: fixel::Point(WINDOW_X_START, slice.0),
            window_end: fixel::Point(WINDOW_X_END, slice.1),
            fixels: array2d::Array2D::filled_with(fixel::Fixel::new(), x as usize, y as usize),
            fixel_size_x: (WINDOW_X_END - WINDOW_X_START)/x as f64,
            fixel_size_y: (slice.1 - slice.0)/y as f64,
            max_target_depth: 64,
            min_achived_depth: 2,
            id: id,
            max_id: max_id
        }
    }

    fn resize(&mut self, x: u32, y: u32){
        // FIXME TODO
        println!("resize fixels");
        let slice = Self::slice(WINDOW_Y_START, WINDOW_Y_END, self.id, self.max_id);
        self.fixels = array2d::Array2D::filled_with(fixel::Fixel::new(), x as usize, y as usize);
        self.fixel_size_x = (WINDOW_X_END - WINDOW_X_START)/x as f64;
        self.fixel_size_y = (slice.1 - slice.0)/y as f64;
    }

    fn get_pixel(&mut self, x: u32, y: u32) -> im::Rgba<u8> {
        match self.fixels[(x as usize, y as usize)].root_type() {
            fixel::RootType::NoRoot => im::Rgba([255,255,255,255]),
            fixel::RootType::Root => im::Rgba([0,0,0,255]),
            fixel::RootType::OutOfDomain => im::Rgba([255,0,0,255])
        }
    }
    fn next_line(&mut self, y: u32){}
    fn next_frame(&mut self){
        if self.min_achived_depth >= self.max_target_depth{
            return;
        }
        self.min_achived_depth += 1; // Issue with resizes, too much roots at once
        for y in 0..self.fixels.row_len(){
            for x in 0..self.fixels.column_len(){
                let (start, end) = self.pixel2fixel(x, y);
                self.fixels[(x, y)].add_samples(equart, &start, &end, self.min_achived_depth);
            }
        }

    }
}

impl Equart{
    /// Convert pixel coordinates to fixel window
    fn pixel2fixel(&self, x: usize, y: usize) -> (fixel::Point, fixel::Point){
        let fixel_start_x = self.window_start.0 + self.fixel_size_x * x as f64;
        let fixel_start_y = self.window_start.1 + self.fixel_size_y * y as f64;
        let fixel_end_x = fixel_start_x + self.fixel_size_x;
        let fixel_end_y = fixel_start_y + self.fixel_size_y;
        return (
            fixel::Point(fixel_start_x, fixel_start_y),
            fixel::Point(fixel_end_x, fixel_end_y)
        );
    }
    fn slice(start: f64, end: f64, id: usize, max_id: usize) -> (f64, f64){
        let span = (end - start)/max_id as f64;
        let begin =  start + span * id as f64;
        let end = begin + span;
        (begin, end)
    }       
}
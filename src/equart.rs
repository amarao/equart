use crate::fixel;
use crate::threads;
use threads::DrawingApp;
use image as im;
const WINDOW_X_START: f64 = -8.0;
const WINDOW_X_END: f64 = 8.0;
const WINDOW_Y_START: f64 = -1.5;
const WINDOW_Y_END: f64 = 1.5;

fn equart(x: f64, y:f64) -> f64{
    (1.0/x).sin() - y
}


pub struct Equart {  // per thread instance, each instance has own 'slice' to work with
    math_start_x: f64,
    math_end_x: f64,
    math_start_y: f64,
    math_end_y: f64,
    fixel_size_x: f64,  // pixel size in math units
    fixel_size_y: f64,  // pixel size in math units
    pixel_size_x: u32,
    pixel_size_y: u32
}

impl DrawingApp for Equart{
    fn new(id: usize, max_id: usize, x: u32, y: u32)->Self {
        let slice = Equart::slice(WINDOW_Y_START, WINDOW_Y_END, id, max_id);
        let mut value = Self{
            math_start_x: WINDOW_X_START,
            math_end_x: WINDOW_X_END,
            math_start_y: slice.0,
            math_end_y: slice.1,
            fixel_size_x: 1.0,
            fixel_size_y: 1.0,
            pixel_size_x: x,
            pixel_size_y: y

        };
        value.update_fixel_size();
        value
        
    }
    fn resize(&mut self, x: u32, y: u32){
        self.pixel_size_x = x;
        self.pixel_size_y = y;
        self.update_fixel_size();
    }

    fn get_pixel(&mut self, x: u32, y: u32) -> im::Rgba<u8> {
        let matrix = self.pixel2matrix(x, y);
        match Equart::is_root(matrix, equart) {
            true => im::Rgba([0, 0, 0, 255]),
            false => im::Rgba([255 ,255, 255, 255]),
        }
    }
}

impl Equart{
    fn is_root<F>(matrix: Vec<[f64;2]>, f: F) -> bool 
    where
        F: Fn(f64, f64) -> f64
    {
        let mut zeroes: bool = false;
        let mut positive: bool = false;
        let mut negative: bool = false;
        for [x,y ] in matrix {
            let res = f(x, y);
            if res == 0.0 {
                zeroes = true;
            } else if res < 0.0 {
                negative = true;
            } else if res > 0.0 {
                positive = true;
            }
            // NaN is skippeed.

        }
        zeroes || (positive && negative)
    }

    fn pixel2matrix(&self, x: u32, y: u32)-> Vec<[f64;2]> {
        // only 2x2 fixel matrix
        let start_x = self.math_start_x + self.fixel_size_x * x as f64;
        let start_y = self.math_start_y + self.fixel_size_y * y as f64;
        vec![
            [start_x, start_y],
            [start_x + self.fixel_size_x, start_y],
            [start_x, start_y + self.fixel_size_y],
            [start_x + self.fixel_size_x, start_y + self.fixel_size_y],
        ]
    }
    fn slice(start: f64, end: f64, id: usize, max_id: usize) -> (f64, f64){
        let span = (end - start)/max_id as f64;
        let begin =  start + span * id as f64;
        let end = begin + span;
        (begin, end)
    }
    fn update_fixel_size(&mut self){
        self.fixel_size_x = (self.math_end_x - self.math_start_x)/self.pixel_size_x as f64;
        self.fixel_size_y = (self.math_end_y - self.math_start_y)/self.pixel_size_y as f64;
    }
}

#[cfg(test)]
mod equart_tests {
    use super::*;
    
    #[test]
    fn is_root_empty(){
        let data_in = vec![];
        assert_eq!(
            Equart::is_root(data_in, |_, __|{0.0}),
            false
        );
    }

    #[test]
    fn is_root_zerores(){
        let data_in = vec![[0.0, 0.0]];
        assert_eq!(
            Equart::is_root(data_in, |_, __|{0.0}),
            true
        );
    }

    #[test]
    fn is_root_positive(){
        let data_in = vec![[0.0, 0.0]];
        assert_eq!(
            Equart::is_root(data_in, |_, __|{1.0}),
            false
        );
    }
    
    #[test]
    fn is_root_negative(){
        let data_in = vec![[0.0, 0.0]];
        assert_eq!(
            Equart::is_root(data_in, |_, __|{-1.0}),
            false
        );
    }
    
    #[test]
    fn is_root_sign_change(){
        let data_in = vec![[-1.0, -1.0], [1.0, 1.0]];
        assert_eq!(
            Equart::is_root(data_in, |x, __|{x}),
            true
        );
    }


    #[test]
    fn slice_one(){
         assert_eq!(Equart::slice(0.0, 1.0, 0, 1), (0.0, 1.0));
    }
    
    #[test]
    fn slice_first_half(){
         assert_eq!(Equart::slice(0.0, 1.0, 0, 2), (0.0, 0.5));
    }
    
    #[test]
    fn slice_second_half(){
         assert_eq!(Equart::slice(0.0, 1.0, 1, 2), (0.5, 1.0));
    }

}
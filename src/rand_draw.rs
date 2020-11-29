use crate::threads;
use threads::DrawingApp;
use image as im;

pub struct RandDraw{
    factor: u64,
    color: [u8;3]
}

impl DrawingApp for RandDraw{
    fn new(id: usize, _max_id: usize, _x: u32, _y: u32)->Self {
        let color_bases = [
            [255, 0, 0],
            [255, 0,255],
            [0, 255,255],
            [255, 255, 0],
            [0, 255, 0],
            [0, 0,255],
            [255, 255, 255]
        ];
        Self{
            factor: 0xFEFABABE,
            color: color_bases[id % color_bases.len()]
        }
    }
    fn get_pixel(&mut self, _x: u32, _y: u32) -> im::Rgba<u8> {
        self.factor ^= self.factor << 13;
        self.factor ^= self.factor >> 17;
        self.factor ^= self.factor << 5;
        let rnd = self.factor.to_be_bytes()[0];
        im::Rgba([
            rnd & self.color[0],
            rnd & self.color[1],
            rnd & self.color[2],
            rnd,
        ])
    }
    
    fn next_line(&mut self, y: u32){}
    fn next_frame(&mut self){}
    fn resize(&mut self, _new_x: u32, _new_y: u32){
    }
}
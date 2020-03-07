pub mod pix {
    use itertools::Itertools;
    pub type Float = f32;
    pub fn sin(x:Float) ->Float {
        x.sin()
    }
    pub fn cos(x:Float) -> Float {
        x.cos()
    }
    #[derive (PartialEq)]
    #[derive (Clone)]
    pub struct Pixel {
        pub index: usize,
    }

    impl  Pixel {
        fn as_pixel(&self, canvas: &Canvas) -> [i64; 2]{
            let row = self.index as i64 / canvas.pixel_x as i64 - canvas.zero_y as i64;
            let column = self.index as i64 %  canvas.pixel_x  as i64 - canvas.zero_x as i64;
            [row, column]
        }
        fn as_cartesian(&self, canvas: &Canvas) -> [Float; 2]{
            let [row, column] = self.as_pixel(canvas);
            let y = (row as Float) * canvas.pixel_size_y;
            let x = (column as Float) * canvas.pixel_size_x;
            [x,  y]
        }
        fn iterate_lattice_as_cartesian(&self, canvas: &Canvas, lattice_dim:usize) -> impl Iterator<Item =[Float;2]> {
            let [x,y] = self.as_cartesian(canvas);
            let (dx, dy) = (canvas.pixel_size_x, canvas.pixel_size_y);
            let subcanvas = (lattice_dim - 1) as Float;
            let conv = move |(i, j): (usize, usize)| {
                    [
                        x+dx/subcanvas * i as Float,
                        y+dy/subcanvas * j as Float
                    ]
                };
            (0..lattice_dim).cartesian_product(0..lattice_dim).map(conv)
        }

        pub fn sign_change_on_lattice<F> (&self, func:F, canvas: &Canvas, lattice_dim:usize) -> bool where
            F: Fn(Float, Float) -> Float
        {
            let mut sign: Option<bool> = None;
            for [x, y] in self.iterate_lattice_as_cartesian(canvas, lattice_dim){
                let res = func(x,y);
                if !res.is_finite() {return false};
                let num_sign = res.signum() > 0.0;
                sign = match sign {
                    None => {Some(num_sign)},
                    Some(old_sign) if old_sign != num_sign => { return true },
                    _ => {continue}
                };
            }
            false
        }
    }

    pub type PixelColor = u8;
    pub struct Canvas {
        pub img: Vec<PixelColor>,
        pub pixel_x: usize,
        pub pixel_y: usize,
        pixel_size_x: Float,
        pixel_size_y: Float,
        zero_x: usize,
        zero_y: usize,
    }

    impl Canvas{
        pub fn new(canvas_x: usize, canvas_y:usize, cartesian_x: Float, cartesian_y:Float, zero_position_x: usize, zero_position_y: usize) -> Canvas {
            let img_size = ((canvas_x as u64)*(canvas_y as u64)) as usize;
            let canvas = Canvas{
                    img: vec![0xFF;img_size],
                    pixel_x: canvas_x,
                    pixel_y: canvas_y,
                    zero_x: zero_position_x,
                    zero_y: zero_position_y,
                    pixel_size_x: cartesian_x/(canvas_x as Float),
                    pixel_size_y: cartesian_y/(canvas_y as Float),
            };
            canvas
        }
        pub fn iter(&self) ->  impl Iterator<Item = Pixel>{
            (0..(self.pixel_x*self.pixel_y)).into_iter().map(|x|Pixel{index: x as usize})
        }


        pub fn get_neighbors(& self, pixel: & Pixel) -> Vec<Pixel>{
            let mut res: Vec<Pixel> = Vec::with_capacity(8);
            for x in -10..11{
                for y in -10..11 {
                    if x==y && y==0 { continue };
                    let neighbor = pixel.index as i64 + y as i64 *self.pixel_x as i64 + x as i64;
                    if neighbor < 0 || neighbor >= self.pixel_x as i64 *self.pixel_y as i64{
                        continue;
                    }
                    res.push(Pixel{index: neighbor as usize});
                }
            }
            res
        }
        pub fn neighbors_roots_count(&self, pixel: &Pixel) -> u64 {
            let mut res = 0;
            for x in -6..7{
                for y in -6..7 {
                    if x==y && y==0 { continue };
                    let neighbor = pixel.index as i64 + y as i64 *self.pixel_x as i64 + x as i64;
                    if neighbor < 0 || neighbor >= self.pixel_x as i64 *self.pixel_y as i64{
                        continue;
                    }
                    if self.img[neighbor as usize] == 0 {res += 1;}
                }
        }
        res
    }

        pub fn set_pixel(& mut self, pixel: &Pixel, value: PixelColor) {
            self.img[pixel.index] = value;
        }
        pub fn get_pixel(&self, pixel:&Pixel) -> PixelColor {
            self.img[pixel.index]
        }
        pub fn roots(&self) -> u64{
            self.img.iter().filter(|&x| *x==0).count() as u64
        }
        pub fn inverted_clone(&self, color: u32) -> Vec<u32>{
            let mut new_img = Vec::with_capacity(self.pixel_x * self.pixel_y);
            for y in 0..self.pixel_y {
                for x in 0..self.pixel_x {
                    new_img.push((self.img[(self.pixel_y - y - 1) * self.pixel_x + x] as u32) * color);
                }
            }
            new_img
        }
    }


}

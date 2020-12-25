// use Array2d;

#[derive(Clone, Copy, Debug, PartialEq)]
/// Describe type of root absence
pub enum Mood{
        NoData,
        Positive,
        Negative
    }
    
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RootType{
    NoRoot,
    Root,
    OutOfDomain
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point(pub f64, pub f64);

/// Return true if point is within a given window
impl Point {
    pub fn in_window(&self, start: &Self, end: &Self) -> bool{
        self.0 >= start.0 &&
        self.0 <= end.0 &&
        self.1 >= start.1 &&
        self.1 <= end.1 
    }
}

pub enum ProbeType{
    ExactRoot,
    Positive,
    Negative,
    OutOfDomain
}

pub struct Probe{
    x: f64,
    y: f64,
    probe_type: ProbeType
}

impl Probe {
    /// Convert real coordinate to pixelated (fixel coordinate)
    fn coord2pos(coord: f64, window_start: f64, window_end: f64, step:f64) -> (Option<usize>, Option<usize>) {
        if coord < window_start || coord > window_end {
            panic!("Coordinate is outside window")
        }
        let non_rounded_pos = (coord - window_start) / step;
        let rounded_pos = non_rounded_pos.trunc();
        let u_pos = rounded_pos as usize;
        let first: Option<usize>;
        let second: Option<usize>;
        if (coord - window_end).abs() <= f64::EPSILON{
            first = None
        }
        else{
            first = Some(u_pos)
        }
        if (non_rounded_pos - rounded_pos).abs() <= f64::EPSILON {
            if u_pos > 0 {
                second = Some(u_pos - 1);
            }
            else{
                second = None
            }
        }else{
            second = None
        }
        (first, second)
    }

    /// Put pair into vec if pair is valid (both are not None)
    fn combine(buf: &mut Vec<[usize;2]>, x:Option<usize>, y:Option<usize>){
        if let (Some(x), Some(y)) = (x, y){
            buf.push([x, y])
        }
    }
    
    /// return list of locations for probe, yielding one or more new fixel coordinates
    pub fn gen_locations(&self, start: Point, end: Point, step_x: f64, step_y: f64) -> Vec<[usize;2]> {
        let mut retval:Vec<[usize;2]> = Vec::new();
        let x_poses = Self::coord2pos(self.x, start.0, end.0, step_x);
        let y_poses = Self::coord2pos(self.y, start.1, end.1, step_y);
        Self::combine(&mut retval, x_poses.0, y_poses.0);
        Self::combine(&mut retval, x_poses.0, y_poses.1);
        Self::combine(&mut retval, x_poses.1, y_poses.0);
        Self::combine(&mut retval, x_poses.1, y_poses.1);
        retval
    }
}

/// Fixel is a area of 'real' mathematical plane with fixel size expressed
/// as two f64 numbers (width and height).
/// For calculation purposes Fixel structure contains only data, needed for
/// calculating if there is a root in a given fixel or not.
/// There are four types of values, each is calculated using relation under
/// investigation:
/// - Exact roots (function is 0 at those coordinates)
/// - Negative points (function is negative at those)
/// - Positive points (function is positive)
/// - Out of domain points (function is out of domain)
/// Root is present in fixel if:
///     - there are non-zero number of roots within fixel
///     - OR there are negative and positive values and no out of domain points.
/// are_roots is cached value and is updated every time anything within fixel is
/// changed.
/// probes is cached value of total number of points in all four categories.
#[derive(Debug,Clone)]
pub struct Fixel {
    exact_roots: Vec<Point>,
    negative: Vec<Point>,
    positive: Vec<Point>,
    out_of_domain: Vec<Point>,
    roots: RootType,
    probes: u32,
    pub mood: Mood,
}

impl Fixel {
    pub fn new() -> Self {
        Fixel{
            exact_roots: Vec::new(),
            negative: Vec::new(),
            positive: Vec::new(),
            out_of_domain: Vec::new(),
            roots: RootType::NoRoot,
            probes: 0,
            mood: Mood::NoData
        }
    }

    pub fn root_type(&self) -> RootType{
        self.roots
    }
    
    fn add_probe<F>(&mut self, point: Point, rel: F)
    where 
        F: FnOnce(f64, f64)->f64
    {
        match rel(point.0, point.1) {
            value if value.is_infinite() => self.out_of_domain.push(point),
            value if value < 0.0 => self.negative.push(point),
            value if value == 0.0 => self.exact_roots.push(point),
            value if value > 0.0 => self.positive.push(point),
            value if value.is_nan() => self.out_of_domain.push(point),
            value => panic!("Bad match for {}", value)
        }
        self.probes += 1;
    }

    pub fn search_roots(&mut self) -> RootType {
        if self.out_of_domain.len() > 0 {
            self.roots = RootType::OutOfDomain;
            self.mood = Mood::NoData;
            return self.roots;
        }
        if self.exact_roots.len() > 0 {
            self.roots = RootType::Root;
            self.mood = Mood::NoData;
            return self.roots;
        }
        
        if self.negative.len() > 0 && self.positive.len() > 0 {
            self.roots = RootType::Root;
            self.mood = Mood::NoData;
            return self.roots;
        }
        if self.negative.len() > 0 {
            self.mood = Mood::Negative;
        }
        if self.positive.len() > 0 {
            self.mood = Mood::Positive;    
        }
        self.roots = RootType::NoRoot;
        return self.roots;
    }

    /// Return if there are any probes in a given window
    fn has_probes(&self, start: &Point, end: &Point) -> bool {
        for probe in self.exact_roots.iter().chain(
            self.positive.iter().chain(
                self.negative.iter().chain(
                    self.out_of_domain.iter()
                )
            )
        ){
            if probe.in_window(start, end){
                return true
            }
        }
        false
    }

    /// Automatically calculate if new probes are needed, and calculate position
    /// of a new probes.
    pub fn add_samples<F>(&mut self,rel: F, start: &Point, end: &Point, expected_probes: u32) -> RootType
        where F: Fn(f64, f64) -> f64
    {
        if self.roots == RootType::Root || self.roots == RootType::OutOfDomain{
            return self.roots;
        }
        
        let mut need_to_place: i64 = expected_probes as i64 - self.probes as i64;
        
        
        // Place dots in the corners first
        let ld = *start;
        let lu = Point(start.0, end.1);
        let rd = Point(end.0, start.1);
        let ru = *end;
        if !self.has_probes(&ld, &ld){
            self.add_probe(ld, &rel);
            need_to_place -= 1;
        }
        if !self.has_probes(&lu, &lu){
            self.add_probe(lu, &rel);
            need_to_place -= 1;
        }
        if !self.has_probes(&rd, &rd){
            self.add_probe(rd, &rel);
            need_to_place -= 1;
        }
        if !self.has_probes(&ru, &ru){
            self.add_probe(ru, &rel);
            need_to_place -= 1;
        }
        if need_to_place <= 0 {
            return self.search_roots();
        }
        let side = (expected_probes as f64).sqrt().ceil() as u32;
        let dx = (end.0 - start.0)/side as f64;
        let dy = (end.1 - start.1)/side as f64;
        for step_x in 0..=side {
            for step_y in 0..=side {
                let win_start = Point(start.0 + step_x as f64 * dx, start.1 + step_y as f64 * dy);
                let win_end = Point(start.0 + (step_x + 1) as f64 * dx, start.1 + (step_y + 1) as f64 * dy);
                if !self.has_probes(&win_start, &win_end){
                    let new_point = Point(
                        start.0 + (step_x as f64 * dx),
                        start.1 + (step_y as f64 * dy)
                    );
                    self.add_probe(new_point, &rel);
                    need_to_place -= 1;
                    if need_to_place == 0 {
                        return self.search_roots();
                    }
                }
            }
        }
        panic!("No place found for {} dots in a fixel from {}x{} to {}x{} ", need_to_place, start.0, start.1, end.0, end.1);
    }
}

impl<'a> IntoIterator for &'a Fixel{
    type Item = Probe;
    type IntoIter = FixelIter<'a>;
    fn  into_iter(self) -> Self::IntoIter {
        FixelIter{
            fixel: self,
            current_queue:RootType::Root,
            idx: 0
        }
    }
}

pub struct FixelIter<'a>{
    fixel: &'a Fixel,
    current_queue: RootType,
    idx: usize
}
impl<'a> Iterator for FixelIter<'a>{
    type Item = Probe;
    fn next(&mut self) -> Option<Self::Item>{
        match self.current_queue{
            RootType::Root => {

                None
            },
            RootType::NoRoot => {
                None
            },
            RootType::OutOfDomain => {
                None
            }
        }
    }
}

mod point_tests {
    use super::*;

    #[test]
    fn in_window_outside(){
        let point = Point(0.0, 0.0);
        assert_eq!(point.in_window(&Point(1.0, 1.0), &Point(2.0, 2.0)), false);
    }

    #[test]
    fn in_window_inside(){
        let point = Point(0.0, 0.0);
        assert!(point.in_window(&Point(-1.0, -1.0), &Point(1.0, 1.0)));
    }

    #[test]
    fn in_window_partial_x(){
        let point = Point(0.0, 0.0);
        assert_eq!(point.in_window(&Point(-1.0, 1.0), &Point(1.0, 2.0)), false);
    }


    #[test]
    fn in_window_partial_y(){
        let point = Point(0.0, 0.0);
        assert_eq!(point.in_window(&Point(1.0, -1.0), &Point(2.0, 1.0)), false);
    }

}

#[cfg(test)]
mod fixel_tests {
    use super::*;
    
    #[test]
    fn fixel_no() {
        let f = Fixel::new();
        assert_eq! (f.root_type(), RootType::NoRoot);
    }

    #[test]
    fn fixel_zero() {
        let mut f = Fixel::new();
        f.add_probe(Point(0.0, 0.0), |_, __| {0.0});
        f.add_probe(Point(0.0, 0.0), |_, __| {-1.0 + -1.0});
        assert_eq! (f.root_type(), RootType::NoRoot);
        f.search_roots();
        assert_eq! (f.root_type(), RootType::Root);
    }

    #[test]
    fn fixel_positive() {
        let mut f = Fixel::new();
        f.add_probe(Point(0.0, 0.0), |_, __| {1.0});
        f.add_probe(Point(0.0, 0.0), |_, __| {1.0});
        f.add_probe(Point(0.0, 0.0), |_, __| {1.0});
        f.add_probe(Point(0.0, 0.0), |_, __| {1.0});
        f.search_roots();
        assert_eq! (f.root_type(), RootType::NoRoot);
    }

    #[test]
    fn fixel_negative() {
        let mut f = Fixel::new();
        f.add_probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.add_probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.add_probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.add_probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.search_roots();
        assert_eq! (f.root_type(), RootType::NoRoot);
    }

    #[test]
    fn fixel_signchange() {
        let mut f = Fixel::new();
        f.add_probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.add_probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.add_probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.add_probe(Point(0.0, 0.0), |_, __| {1.0});
        f.search_roots();
        assert_eq! (f.root_type(), RootType::Root);
    }

    #[test]
    fn fixel_nan() {
        let mut f = Fixel::new();
        f.add_probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.add_probe(Point(0.0, 0.0), |_, __| {1.0});
        f.add_probe(Point(0.0, 0.0), |_, __| {std::f64::NAN});
        f.search_roots();
        assert_eq! (f.root_type(), RootType::OutOfDomain);
    }

}


#[cfg(test)]
mod probe_tests {
    use super::*;

    #[test]
    fn coord2pos_normal_1(){
        assert_eq!(
            Probe::coord2pos(0.5, -1.0, 1.0, 1.0),
            (Some(1), None)
        )
    }

    #[test]
    fn coord2pos_normal_2(){
        assert_eq!(
            Probe::coord2pos(-0.5, -1.0, 1.0, 1.0),
            (Some(0), None)
        )
    }

    /// Avoid -1 in position
    #[test]
    fn coord2pos_case1(){
        assert_eq!(
            Probe::coord2pos(-1.0, -1.0, 1.0, 1.0),
            (Some(0), None)
        )
    }

    /// Avoid overflow for last pixel
    #[test]
    fn coord2pos_case2(){
        assert_eq!(
            Probe::coord2pos(1.0, -1.0, 1.0, 1.0),
            (None, Some(1))
        )
    }

    /// Get both positions for edge coordinates
    #[test]
    fn coord2pos_case3(){
        assert_eq!(
            Probe::coord2pos(0.0, -1.0, 1.0, 1.0),
            (Some(1), Some(0))
        )
    }
    
    #[test]
    #[should_panic]
    fn coord2pos_panic_under(){
        Probe::coord2pos(-1.1, -1.0, 1.0, 1.0);
    }

    #[test]
    #[should_panic]
    fn coord2pos_panic_over(){
        Probe::coord2pos(1.1, -1.0, 1.0, 1.0);
    }

    #[test]
    fn gen_locations_simple(){
        let p = Probe{x:0.5, y:0.5, probe_type: ProbeType::ExactRoot};
        assert_eq!(
            p.gen_locations(
                Point(-1.0, -1.0),
                Point(1.0, 1.0),
                1.0, 1.0
            ),
            vec![[1, 1]]
        )
    }
    #[test]
    fn gen_locations_worst_corner(){
        let p = Probe{x:1.0, y:1.0, probe_type: ProbeType::ExactRoot};
        assert_eq!(
            p.gen_locations(
                Point(-1.0, -1.0),
                Point(1.0, 1.0),
                1.0, 1.0
            ),
            vec![[1, 1]]
        )
    }
    #[test]
    fn gen_locations_mid_corner(){
        let p = Probe{x:0.0, y:0.0, probe_type: ProbeType::ExactRoot};
        assert_eq!(
            p.gen_locations(
                Point(-1.0, -1.0),
                Point(1.0, 1.0),
                1.0, 1.0
            ),
            vec![[1, 1], [1, 0], [0, 1], [0, 0]]
        )
    }
}
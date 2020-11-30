#[derive(Clone, Copy, Debug, PartialEq)]
/// Describe type of root absence
pub enum RootMood{
    NoData,
    Positive,
    Negative
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RootType{
    NoRoot(RootMood),
    Root,
    OutOfDomain
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point(pub f64, pub f64);


/// Return true if point is within a given window
impl Point {
    pub fn in_window(&self, start: &Self, end: &Self) -> bool{
        self.0 > start.0 &&
        self.0 <= end.0 &&
        self.1 > start.1 &&
        self.1 <= end.1 
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
    pub exact_roots: Vec<Point>,
    pub negative: Vec<Point>,
    pub positive: Vec<Point>,
    pub out_of_domain: Vec<Point>,
    pub roots: RootType,
    pub x_neighbor: RootType,
    pub y_neighbor: RootType,
    pub probes: u32
}

impl Fixel {
    pub fn new() -> Self {
        Fixel{
            exact_roots: Vec::new(),
            negative: Vec::new(),
            positive: Vec::new(),
            out_of_domain: Vec::new(),
            roots: RootType::NoRoot(RootMood::NoData),
            x_neighbor: RootType::NoRoot(RootMood::NoData),
            y_neighbor: RootType::NoRoot(RootMood::NoData),
            probes: 0
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
            value if value < 0.0 => self.negative.push(point),
            value if value == 0.0 => self.exact_roots.push(point),
            value if value > 0.0 => self.positive.push(point),
            value if value.is_nan() => self.out_of_domain.push(point),
            value => panic!("Bad match for {}", value)
        }
        self.probes += 1;
    }

    fn search_roots(&mut self) -> RootType {
        if self.out_of_domain.len() > 0 {
            self.roots = RootType::OutOfDomain;
            return RootType::OutOfDomain;
        }
        if self.exact_roots.len() > 0 {
            self.roots = RootType::Root;
            return RootType::Root;
        }
        if self.negative.len() > 0 && self.positive.len() > 0 {
            self.roots = RootType::Root;
            return RootType::Root;
        }
        if self.negative.len() > 0 {
            self.roots = RootType::NoRoot(RootMood::Negative);
            return RootType::NoRoot(RootMood::Negative);
        }
        if self.positive.len() > 0 {
            self.roots = RootType::NoRoot(RootMood::Positive);
            return RootType::NoRoot(RootMood::Positive);
        }
        self.roots = RootType::NoRoot(RootMood::NoData);
        return RootType::NoRoot(RootMood::NoData);
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
    pub fn add_samples<F>(&mut self,rel: F, start: &Point, end: &Point, expected_probes: u32, x_neighbor: RootType, y_neighbor: RootType) -> RootType
        where F: Fn(f64, f64) -> f64
    {
        self.x_neighbor = x_neighbor;
        self.y_neighbor = y_neighbor;
        if self.probes >= expected_probes {
            return self.root_type();
        }
        let side = (expected_probes as f64).sqrt().ceil() as u32;
        let need_to_place = expected_probes - self.probes;
        let mut countdown = need_to_place;
        let dx = (end.0 - start.0)/side as f64;
        let dy = (end.1 - start.1)/side as f64;
        for step_x in 0..side {
            for step_y in 0..side {
                let win_start = Point(start.0 + step_x as f64 * dx, start.1 + step_y as f64 * dy);
                let win_end = Point(start.0 + (step_x + 1) as f64 * dx, start.1 + (step_y + 1) as f64 * dy);
                if !self.has_probes(&win_start, &win_end){
                    let new_point = Point(
                        start.0 + (step_x as f64 * dx),
                        start.1 + (step_y as f64 * dy)
                    );
                    self.add_probe(new_point, &rel);
                    countdown -= 1;
                    if countdown == 0 {
                        return self.search_roots();
                    }
                }
            }
        }
        panic!("No place found for {} dots in a fixel from {}x{} to {}x{} ", countdown, start.0, start.1, end.0, end.1);
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
        f.add_probe(Point(0.0, 0.0), |_, __| {1.0 - 1.0});
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

    #[test]
    fn add_samples_trivial() {
        let mut f = Fixel::new();
        assert_eq!(f.add_samples(|_, __| {0.0}, &Point(-1.0, -1.0), &Point(1.0, 1.0), 2, RootType::NoRoot(RootMood::NoData), RootType::NoRoot(RootMood::NoData)), 2);
        assert_eq!(f.probes, 2);
    }

    #[test]
    fn add_samples_next() {
        let mut f = Fixel::new();
        f.add_samples(|_, __| {0.0}, &Point(-1.0, -1.0), &Point(1.0, 1.0), 4, RootType::NoRoot(RootMood::NoData), RootType::NoRoot(RootMood::NoData));
        f.add_samples(|_, __| {0.0}, &Point(-1.0, -1.0), &Point(1.0, 1.0), 13, RootType::NoRoot(RootMood::NoData), RootType::NoRoot(RootMood::NoData));
        assert_eq!(f.probes, 13);
    }

    #[test]
    fn add_samples_uniqueness() {
        let mut points: Vec<Point> = Vec::new();
        let mut f = Fixel::new();
        for x in 2..15{
            f.add_samples(|_, __| {0.0}, &Point(-1.0, -1.0), &Point(1.0, 1.0), x, RootType::NoRoot(RootMood::NoData), RootType::NoRoot(RootMood::NoData));
        }
        for probe in f.exact_roots.iter().chain(
            f.positive.iter().chain(
                f.negative.iter().chain(
                    f.out_of_domain.iter()
                )
            )
        ){
            println!("{:?}", probe);
            assert!(!points.contains(probe));
            points.push(*probe);
        }
    }

    #[test]
    fn add_samples_are_in_range() {
        let mut f = Fixel::new();
        let start = Point(-1.0, -1.0);
        let end = Point(1.0, 1.0);
        f.add_samples(|_, __| {0.0}, &start, &end, 13, RootType::NoRoot(RootMood::NoData), RootType::NoRoot(RootMood::NoData));
        for probe in f.exact_roots.iter().chain(
            f.positive.iter().chain(
                f.negative.iter().chain(
                    f.out_of_domain.iter()
                )
            )
        ){
            assert!(probe.in_window(&start, &end));
        }
    }
}


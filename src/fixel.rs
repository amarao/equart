#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RootType{
    NoRoot,
    Root,
    OutOfDomain
}

#[derive(Debug, Clone, Copy)]
pub struct Point(f64, f64);

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
#[derive(Debug)]
pub struct Fixel {
    exact_roots: Vec<Point>,
    negative: Vec<Point>,
    positive: Vec<Point>,
    out_of_domain: Vec<Point>,
    roots: RootType,
    probes: u32
}

impl Fixel {
    fn new() -> Self {
        Fixel{
            exact_roots: Vec::new(),
            negative: Vec::new(),
            positive: Vec::new(),
            out_of_domain: Vec::new(),
            roots: RootType::NoRoot,
            probes: 0
        }
    }

    fn root_type(&self) -> RootType{
        self.roots
    }
    
    fn probe<F>(&mut self, point: Point, rel: F)
    where 
        F: Fn(f64, f64)->f64
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

    fn update_type(&mut self) {
        if self.out_of_domain.len() > 0 {
            self.roots = RootType::OutOfDomain;
            return;
        }
        if self.exact_roots.len() > 0 {
            self.roots = RootType::Root;
            return;
        }
        if self.negative.len() > 0 && self.positive.len() > 0 {
            self.roots = RootType::Root
        }else {
            self.roots = RootType::NoRoot
        }
    }

    /// Automatically calculate if new probes are needed, and calculate position
    /// of a new probes.
    fn sample<F>(&mut self,rel: F, start: Point, end: Point , expected_probes: u32) -> u32
        where F: Fn(f64, f64) -> f64
    {
        if self.probes >= expected_probes {
            return 0
        }
        return 1

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
        f.probe(Point(0.0, 0.0), |_, __| {1.0 - 1.0});
        f.probe(Point(0.0, 0.0), |_, __| {-1.0 + -1.0});
        assert_eq! (f.root_type(), RootType::NoRoot);
        f.update_type();
        assert_eq! (f.root_type(), RootType::Root);
    }

    #[test]
    fn fixel_positive() {
        let mut f = Fixel::new();
        f.probe(Point(0.0, 0.0), |_, __| {1.0});
        f.probe(Point(0.0, 0.0), |_, __| {1.0});
        f.probe(Point(0.0, 0.0), |_, __| {1.0});
        f.probe(Point(0.0, 0.0), |_, __| {1.0});
        f.update_type();
        assert_eq! (f.root_type(), RootType::NoRoot);
    }

    #[test]
    fn fixel_negative() {
        let mut f = Fixel::new();
        f.probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.update_type();
        assert_eq! (f.root_type(), RootType::NoRoot);
    }


    #[test]
    fn fixel_signchange() {
        let mut f = Fixel::new();
        f.probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.probe(Point(0.0, 0.0), |_, __| {1.0});
        f.update_type();
        assert_eq! (f.root_type(), RootType::Root);
    }

    #[test]
    fn fixel_nan() {
        let mut f = Fixel::new();
        f.probe(Point(0.0, 0.0), |_, __| {-1.0});
        f.probe(Point(0.0, 0.0), |_, __| {1.0});
        f.probe(Point(0.0, 0.0), |_, __| {std::f64::NAN});
        f.update_type();
        assert_eq! (f.root_type(), RootType::OutOfDomain);
    }
}
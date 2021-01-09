// const N: usize = 4;
#[derive(Debug,Clone,Copy)]
pub struct Point {
    x: f64,
    y: f64
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point{x,y}
    }

    pub fn in_area(&self, start: Point, end: Point) -> bool {
        self.x >= start.x && self.x <= end.x && self.y >= start.y && self.y <= end.y
    }

}

impl PartialEq for Point{
    fn eq(&self, other: &Self) -> bool{
        (self.x - other.x).abs() < f64::EPSILON &&
        (self.y - other.y).abs() < f64::EPSILON
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Boundry{
    start: Point,
    end: Point
}

impl Boundry {
    pub fn new(start: Point, end: Point) -> Self{
        Boundry {
            start: start,
            end: end
        }
    }

    pub fn from_coords(start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Self{
        Self::new(Point::new(start_x, start_y), Point::new(end_x, end_y))
    }

    pub fn is_inside(&self, p: Point) -> bool {
        p.in_area(self.start, self.end)
    }

    pub fn split(&self) -> [Self;4]{
        let len_x = (self.end.x - self.start.x) / 2.0;
        let len_y = (self.end.y - self.start.y) / 2.0;
        [
            Self::from_coords(self.start.x, self.start.y, self.start.x + len_x, self.start.y + len_y),
            Self::from_coords(self.start.x + len_x, self.start.y, self.end.x, self.start.y + len_y),
            Self::from_coords(self.start.x, self.start.y + len_y, self.start.x + len_x, self.end.y),
            Self::from_coords(self.start.x + len_x, self.start.y + len_y, self.end.x, self.end.y),
        ]
    }

    pub fn find_quadrant(&self, p: Point) -> (Self, usize){
        let quadrants = self.split();
        for (i, q) in quadrants.iter().enumerate(){
            if q.is_inside(p){
                return (*q, i)
            }
        }
        panic!("point {:?} outside of boundries {:?}", p, self);
    }

    /// Return true if other overlaps with self.
    pub fn overlaps(&self, other: Self) -> bool {
        self.start.in_area(other.start, other.end) ||
        self.end.in_area(other.start, other.end) ||
        other.start.in_area(self.start, self.end) ||
        other.end.in_area(self.start, self.end)
    }
}

impl PartialEq for Boundry{
    fn eq(&self, other: &Self) -> bool{
        self.start == other.start && self.end == other.end
    }
}

const MAX_POINTS: usize = 10;

enum QuadTreeNode<T> {
    Leaf(Point, T),
    Node(Vec<QuadTreeNode<T>>),
    PointGroup(Vec<(Point, T)>),
    None
}

pub struct QuadTree<T>{
    boundry: Boundry,
    node: QuadTreeNode<T>,
}

impl<T> QuadTree<T>{
    pub fn new(b: Boundry) -> Self{
        QuadTree{
            boundry: b,
            node: QuadTreeNode::None
        }
    }

    pub fn is_inside(&self, p: Point) -> bool{
        self.boundry.is_inside(p)
    }

    pub fn append_point(&mut self, coords: Point, data: T) -> Result<(), ()>{
        if !self.boundry.is_inside(coords){
            return Err(());
        }
        self.node.append_point(self.boundry, coords, data);
        Ok(())
    }
    /// Search data py point
    pub fn search(&self, p: Point) -> Option<&T>{
        if !self.boundry.is_inside(p){
            return None;
        }
        self.node.search(self.boundry, p)
    }
    
    pub fn values_in_area(&self, search_area: Boundry) -> Vec<&T>{
        self.node.values_in_area(self.boundry, search_area)
    }
}

impl<T> Default for QuadTreeNode<T>{
    fn default()-> Self{
        QuadTreeNode::None
    }
}

impl<T> QuadTreeNode<T>{
    fn is_none(&self) -> bool {
        match self{
            QuadTreeNode::None => true,
            _ => false,
        }
    }
    fn append_point(&mut self, boundry: Boundry, coords: Point, data: T) {
        // let newnode = QuadTreeNode::None;
        // let oldnode = std::mem::replace(&mut self.node, newnode);
        let mut current = std::mem::take(self);
        match current {
            QuadTreeNode::None => {
                *self = QuadTreeNode::Leaf(coords, data);
            },
            QuadTreeNode::Leaf(old_coords, old_data) => {
                if old_coords != coords{
                    let mut v = Vec::with_capacity(MAX_POINTS);
                    v.push((old_coords, old_data));
                    v.push((coords, data));
                    *self = QuadTreeNode::PointGroup(v);
                }
                else{
                    *self = QuadTreeNode::Leaf(coords, data);
                }
            },
            QuadTreeNode::PointGroup(mut point_vec) => {
                for i in 0..point_vec.len(){
                    if point_vec[i].0 == coords{
                        point_vec[i] = (coords, data);
                        * self = QuadTreeNode::PointGroup(point_vec);
                        return;
                    }
                }
                if point_vec.len() < MAX_POINTS{
                    point_vec.push((coords, data));
                    * self = QuadTreeNode::PointGroup(point_vec);
                }
                else {
                    let mut quadrants = vec![QuadTreeNode::None, QuadTreeNode::None, QuadTreeNode::None, QuadTreeNode::None];
                    for (old_coords, old_data) in point_vec{
                        let (subboundry, index) = boundry.find_quadrant(old_coords);
                        quadrants[index].append_point(subboundry, old_coords, old_data);
                    }
                    let (subboundry, index) = boundry.find_quadrant(coords);
                    quadrants[index].append_point(subboundry, coords, data);
                    * self = QuadTreeNode::Node(quadrants);
                }
            }
            QuadTreeNode::Node(ref mut quadrants) => {
                let (subboundry, index) = boundry.find_quadrant(coords);
                if quadrants[index].is_none(){
                    quadrants[index] = QuadTreeNode::Leaf(coords, data);
                }else{
                    quadrants[index].append_point(subboundry, coords, data);
                }
                *self = current;
            }
        }
    }
    fn search(&self, b: Boundry, p: Point) -> Option<&T>{
        match self{
            QuadTreeNode::None => None,
            QuadTreeNode::Leaf(coords, data) => {
                if *coords == p{
                    Some(&data)
                }
                else {
                    None
                }
            },
            QuadTreeNode::PointGroup(point_vec) =>{
                for i in 0..point_vec.len(){
                    if point_vec[i].0 == p{
                        return Some(&point_vec[i].1)
                    }
                }
                None
            },
            QuadTreeNode::Node(quadrants) => {
                let (subboundry, index) = b.find_quadrant(p);
                quadrants[index].search(subboundry, p)
            }
        }
    }

    fn values_in_area(&self, own_area: Boundry, search_area: Boundry) -> Vec<&T>{
        let mut found = Vec::new();
        match self{
            QuadTreeNode::None => {},
            QuadTreeNode::Leaf(coords, data) => {
                if search_area.is_inside(*coords){
                    found.push(data);
                }
            }
            QuadTreeNode::PointGroup(point_vec) => {
                for i in 0..point_vec.len(){
                    if search_area.is_inside(point_vec[i].0){
                        found.push(&point_vec[i].1);
                    }
                }
            }
            QuadTreeNode::Node(quadrants) => {
                let own_subareas = own_area.split();
                for i in 0..4{
                    if own_subareas[i].overlaps(search_area) {
                        found.append(&mut quadrants[i].values_in_area(own_subareas[i], search_area));
                    }
                }
            }
        }
        found
    }
}
            

#[cfg(test)]
mod test_quadtree{
    use super::*;

    #[test]
    fn point_eq() {
        let one = Point { x: 2.0, y: 1.0 };
        let two = Point { x: 2.0, y: 1.0 };
        assert_eq!(one, two);
    }

    #[test]
    fn boundry_eq() {
        let one = Boundry { start: Point { x: 1.0, y: 0.0 }, end: Point { x: 2.0, y: 1.0 } };
        let two = Boundry { start: Point { x: 1.0, y: 0.0 }, end: Point { x: 2.0, y: 1.0 } };
        assert_eq!(one, two);
    }

    #[test]
    fn boundry_split() {
        let input = Boundry::from_coords(0.0, 0.0, 2.0, 2.0).split();
        assert_eq!(
            input[0],
            Boundry::from_coords(0.0, 0.0, 1.0, 1.0)
        );
        assert_eq!(
            input[1],
            Boundry::from_coords(1.0, 0.0, 2.0, 1.0)
        );
    }

    #[test]
    fn overlaps_inside(){
        let area_one = Boundry::from_coords(0.0, -50.0, 100.0, -10.0);
        let area_two = Boundry::from_coords(1.0, -20.0, 2.0, -19.0);
        assert!(area_one.overlaps(area_two));
        assert!(area_two.overlaps(area_one));
    }

    #[test]
    fn overlaps_matching(){
        let area_one = Boundry::from_coords(0.0, -50.0, 100.0, -10.0);
        assert!(area_one.overlaps(area_one));
    }

    #[test]
    fn overlaps_partial(){
        let area_one = Boundry::from_coords(0.0, 0.0, 2.0, 2.0);
        let area_two = Boundry::from_coords(1.0, 1.0, 3.0, 3.0);
        assert!(area_one.overlaps(area_two));
        assert!(area_two.overlaps(area_one));
    }

    #[test]
    fn overlaps_edge_1(){
        let area_one = Boundry::from_coords(0.0, 0.0, 1.0, 1.0);
        let area_two = Boundry::from_coords(1.0, 1.0, 3.0, 3.0);
        assert!(area_one.overlaps(area_two));
        assert!(area_two.overlaps(area_one));
    }

    #[test]
    fn overlaps_edge_2(){
        let area_one = Boundry::from_coords(0.0, 0.0, 1.0, 1.0);
        let area_two = Boundry::from_coords(1.0, 0.0, 3.0, 3.0);
        assert!(area_one.overlaps(area_two));
        assert!(area_two.overlaps(area_one));
    }

    #[test]
    fn overlaps_disjoint(){
        let area_one = Boundry::from_coords(0.0, 0.0, 1.0, 1.0);
        let area_two = Boundry::from_coords(2.0, 2.0, 3.0, 3.0);
        assert!(!area_one.overlaps(area_two));
        assert!(!area_two.overlaps(area_one));
    }


    #[test]
    fn append_search_normal() {
        let mut foo = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        let point = Point::new(0.0, 0.0);
        assert_eq!(foo.append_point(point, 42), Ok(()));
        assert_eq!(foo.search(point), Some(&42));
    }
    #[test]
    fn append_oob() {
        let mut foo = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        let point = Point::new(42.0, 42.0);
        assert_eq!(foo.append_point(point, 42), Err(()));
    }
    #[test]
    fn not_found() {
        let mut foo = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        let point = Point::new(0.0, 0.0);
        let other_point = Point::new(0.1, 0.1);
        foo.append_point(point, 42).unwrap();
        assert_eq!(foo.search(other_point), None);
    }

    #[test]
    fn add_one() {
        let mut tree = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        let point1 = Point::new(0.1, 0.1);
        assert_eq!(tree.append_point(point1, 1), Ok(()));
        assert_eq!(tree.search(point1), Some(&1));
    }

    #[test]
    fn add_two() {
        let mut tree = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        let point1 = Point::new(0.1, 0.1);
        let point2 = Point::new(0.2, 0.2);
        assert_eq!(tree.append_point(point1, 1), Ok(()));
        assert_eq!(tree.append_point(point2, 2), Ok(()));
        assert_eq!(tree.search(point1), Some(&1));
        assert_eq!(tree.search(point2), Some(&2));
    }


    #[test]
    fn add_four_interleave() {
        let mut tree = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        let point1 = Point::new(0.1, 0.1);
        let point2 = Point::new(0.2, 0.2);
        let point3 = Point::new(0.3, 0.3);
        let point4 = Point::new(0.4, 0.4);
        assert_eq!(tree.append_point(point1, 1), Ok(()));
        assert_eq!(tree.search(point1), Some(&1));
        assert_eq!(tree.append_point(point2, 2), Ok(()));
        assert_eq!(tree.search(point1), Some(&1));
        assert_eq!(tree.search(point2), Some(&2));
        assert_eq!(tree.append_point(point3, 3), Ok(()));
        assert_eq!(tree.search(point1), Some(&1));
        assert_eq!(tree.search(point2), Some(&2));
        assert_eq!(tree.search(point3), Some(&3));
        assert_eq!(tree.append_point(point4, 4), Ok(()));
        assert_eq!(tree.search(point4), Some(&4));
    }

    #[test]
    fn add_four() {
        let mut tree = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        let point1 = Point::new(0.1, 0.1);
        let point2 = Point::new(0.2, 0.2);
        let point3 = Point::new(0.3, 0.3);
        let point4 = Point::new(0.4, 0.4);
        assert_eq!(tree.append_point(point1, 1), Ok(()));
        assert_eq!(tree.append_point(point2, 2), Ok(()));
        assert_eq!(tree.append_point(point3, 3), Ok(()));
        assert_eq!(tree.append_point(point4, 4), Ok(()));
        assert_eq!(tree.search(point1), Some(&1));
        assert_eq!(tree.search(point2), Some(&2));
        assert_eq!(tree.search(point3), Some(&3));
        assert_eq!(tree.search(point4), Some(&4));
    }

    #[test]
    fn fill_quadrant() {
        let mut tree = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        let mut point = Point::new(0.5, 0.5);
        for cnt in 0..1024{
            point.x /= 1.01;
            point.y /= 1.01;
            assert_eq!(tree.append_point(point, cnt), Ok(()));
            assert_eq!(tree.search(point), Some(&cnt));
        }
    }
    
    #[test]
    fn values_in_area_empty() {
        let tree: QuadTree<(f64, f64)> = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        let search_area = Boundry::from_coords(-1.0, -1.0, 1.0, 1.0);
        assert_eq!(tree.values_in_area(search_area).len(), 0);
    }

    #[test]
    fn values_in_area_one() {
        let mut tree = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        let point = Point::new(0.0, 0.0);
        tree.append_point(point, 42.0).unwrap();
        let search_area = Boundry::from_coords(-1.0, -1.0, 1.0, 1.0);
        assert_eq!(tree.values_in_area(search_area).len(), 1);
    }

    #[test]
    fn values_in_area_deep() {
        let mut tree = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        for i in 0..100 {
            let point = Point::new(i as f64/1000.0, i as f64/1000.0);
            tree.append_point(point, i).unwrap();
        }
        let search_area = Boundry::from_coords(-1.0, -1.0, 1.0, 1.0);
        assert_eq!(tree.values_in_area(search_area).len(), 100);
    }

    #[test]
    fn values_in_area_wide_with_overlap() {
        let mut tree = QuadTree::new(Boundry::from_coords(0.0, 0.0, 1.0, 1.0));
        for i in 0..100 {
            let point = Point::new(i as f64/100.0, i as f64/100.0);
            tree.append_point(point, i).unwrap();
        }
        let search_area = Boundry::from_coords(-1.0, -1.0, 2.0, 2.0);
        assert_eq!(tree.values_in_area(search_area).len(), 100);
    }

    #[test]
    fn values_in_area_partial() {
        let mut tree = QuadTree::new(Boundry::from_coords(0.0, 0.0, 1.0, 1.0));
        for i in 0..100 {
            let point = Point::new(i as f64/100.0, i as f64/100.0);
            tree.append_point(point, i).unwrap();
        }
        let search_area = Boundry::from_coords(0.0, 0.0, 0.1, 0.1);
        // we have 11 results because area is inclusive 0.1, 0.1 is match
        // as well as 0.0
        assert_eq!(tree.values_in_area(search_area).len(), 11);
    }

    #[test]
    fn values_in_area_outside_search_range() {
        let mut tree = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        for i in 0..100 {
            let point = Point::new(i as f64/100.0, i as f64/100.0);
            tree.append_point(point, i).unwrap();
        }
        let search_area = Boundry::from_coords(-1.0, -1.0, -0.5, -0.5);
        assert_eq!(tree.values_in_area(search_area).len(), 0);
    }

    #[test]
    fn values_in_area_small_range() {
        let mut tree = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        for i in 0..100 {
            let point = Point::new(i as f64/100.0, i as f64/100.0);
            tree.append_point(point, i).unwrap();
        }
        let search_area = Boundry::from_coords(0.895, 0.895, 0.905, 0.905);
        assert_eq!(tree.values_in_area(search_area).len(), 1);
    }

    #[test]
    fn values_in_area_too_small_range() {
        let mut tree = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        for i in 0..100 {
            let point = Point::new(i as f64/100.0, i as f64/100.0);
            tree.append_point(point, i).unwrap();
        }
        let search_area = Boundry::from_coords(0.898, 0.898, 0.899, 0.899);
        assert_eq!(tree.values_in_area(search_area).len(), 0);
    }

    #[test]
    fn good_fill() {
        let mut q = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        for x in -1000..1000{
            for y in -1000..1000{
                let p = Point::new(x as f64/1000.0, y as f64/1000.0);
                q.append_point(p, (x,y)).unwrap();
            }
        }
        assert!(q.search(Point::new(0.0, 0.0)).is_some());
    }
        

    #[test]
    fn double_fill(){
        // stack overflow from benchmark function
        fn black_box<T>(x:T) ->T {
            x
        }
        let mut q = QuadTree::new(Boundry::from_coords(0.0, 0.0, 20.0, 20.0));
        for c in 0..2{
            for x in 0..20{
                for y in 0..20{
                    let p = Point::new(black_box(x as f64), black_box(y as f64));
                    black_box(q.append_point(p, black_box(x+y)));
                }
            }
        }
    }
}

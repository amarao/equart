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

    pub fn in_range(&self, start: Point, end: Point) -> bool {
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
        p.in_range(self.start, self.end)
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
        for i in 0..4{
            if quadrants[i].is_inside(p){
                return (quadrants[i], i)
            }
        }
        panic!("point {:?} outside of boundries {:?}", p, self);
    }
}

impl PartialEq for Boundry{
    fn eq(&self, other: &Self) -> bool{
        self.start == other.start && self.end == other.end
    }
}

enum QuadTreeNode<T> {
    Leaf(Point, T),
    Node([Option<Box<QuadTreeNode<T>>>; 4]),
    None
}

struct QuadTree<T>{
    boundry: Boundry,
    node: QuadTreeNode<T>,
}

impl<T> QuadTree<T>{
    fn new(b: Boundry) -> Self{
        QuadTree{
            boundry: b,
            node: QuadTreeNode::None
        }
    }

    // fn new_with_data(b: Boundry) -> Self{
    //     QuadTree{
    //         boundry: b,
    //         node: QuadTreeNode::None
    //     }
    // }

    fn is_inside(&self, p: Point) -> bool{
        self.boundry.is_inside(p)
    }

    fn append_point(&mut self, coords: Point, data: T) -> Result<(), ()>{
        if !self.boundry.is_inside(coords){
            return Err(());
        }
        self.node.append_point(self.boundry, coords, data);
        Ok(())
    }
    /// Search data py point
    fn search(&self, p: Point) -> Option<&T>{
        if !self.boundry.is_inside(p){
            return None;
        }
        self.node.search(self.boundry, p)
    }
}
impl<T> QuadTreeNode<T>{
    fn append_point(&mut self, boundry: Boundry, coords: Point, data: T) {
        // let newnode = QuadTreeNode::None;
        // let oldnode = std::mem::replace(&mut self.node, newnode);
        let stub = QuadTreeNode::None;
        let mut current = std::mem::replace(self, stub);
        match current {
            QuadTreeNode::None => {
                std::mem::replace(self, QuadTreeNode::Leaf(coords, data));
            },
            QuadTreeNode::Leaf(old_coords, old_data) => {
                if old_coords != coords {
                    let (subboundry, index) = boundry.find_quadrant(coords);
                    let (old_subboundry, old_index) = boundry.find_quadrant(old_coords);
                    let mut node = [None, None, None, None];
                    let mut newnode = QuadTreeNode::Leaf(coords, data);
                    if index == old_index {
                        newnode.append_point(old_subboundry, old_coords, old_data);
                    }else{
                        let oldnode = QuadTreeNode::Leaf(old_coords, old_data);
                        node[old_index] = Some(Box::new(oldnode));
                    }
                    node[index] =  Some(Box::new(newnode));
                    std::mem::replace(self, QuadTreeNode::Node(node));
                }
            },
            QuadTreeNode::Node(ref mut quadrants) => {
                let (subboundry, index) = boundry.find_quadrant(coords);
                if quadrants[index].is_none(){
                    quadrants[index] = Some(Box::new(QuadTreeNode::Leaf(coords, data)));
                }else{
                    let mut some_box_subnode = None;
                    std::mem::swap(&mut quadrants[index], &mut some_box_subnode);
                    let mut subnode = *(some_box_subnode.unwrap());
                    subnode.append_point(subboundry, coords, data);
                    quadrants[index] = Some(Box::new(subnode));
                }
                std::mem::replace(self, current);

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
            QuadTreeNode::Node(quadrants) => {
                let (subboundry, index) = b.find_quadrant(p);
                if quadrants[index].is_none(){
                    None
                }
                else{
                    let unpacked = quadrants[index].as_ref().unwrap();
                    unpacked.search(subboundry, p)
                }
            }
        }
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
    fn fill_quadrant() {
        let mut foo = QuadTree::new(Boundry::from_coords(-1.0, -1.0, 1.0, 1.0));
        let mut point = Point::new(0.5, 0.5);
        for cnt in 0..1024{
            point.x /= 1.01;
            point.y /= 1.01;
            assert_eq!(foo.append_point(point, cnt), Ok(()));
            assert_eq!(foo.search(point), Some(&cnt));
        }
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
    }
}

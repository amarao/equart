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

    pub fn x_in_range(&self, start: Point, end: Point) -> bool {
        self.x >= start.x && self.x <= end.x && self.y >= start.y && self.y <= end.y
    }

}

impl PartialEq for Point{
    fn eq(&self, other: &Self) -> bool{
        (self.x - other.x).abs() < f64::EPSILON &&
        (self.y - other.y).abs() < f64::EPSILON
    }
}

#[derive(Debug)]
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
        p.x_in_range(self.start, self.end)
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
}

impl PartialEq for Boundry{
    fn eq(&self, other: &Self) -> bool{
        self.start == other.start && self.end == other.end
    }
}

enum QuadTreeNode<T> {
    Leaf(Point, T),
    Node([Box<QuadTree<T>>; 4]),
    None
}

struct QuadTree<T>{
    node: QuadTreeNode<T>,
    boundry: Boundry,
}

impl<T> QuadTree<T>{
    fn new(b: Boundry) -> Self{
        QuadTree{
            boundry: b,
            node: QuadTreeNode::None
        }
    }

    fn is_inside(&self, p: Point) -> bool{
        self.boundry.is_inside(p)
    }

    fn append_point(&mut self, coords: Point, data: T) -> Result<(), &str>{
        if !self.boundry.is_inside(coords){
            return Err("Point outside of boundries");
        }
        match &mut self.node{
            QuadTreeNode::None => {
                // self.node = QuadTreeNode::Leaf(coords, data);
                Ok(())
            }
            QuadTreeNode::Node(quadrants) => {
                for quadrant in quadrants.iter_mut(){
                    if quadrant.is_inside(coords){
                        return quadrant.append_point(coords, data);
                    }
                }
                Err("Point is not in any of quardrants")
            }
            _ => {
                Err("")
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
}
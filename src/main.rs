extern crate turtle;

use turtle::Turtle;

trait Point {
    fn point(&mut self, x:f64, y:f64);
}

impl Point for Turtle {
    fn point(&mut self, x:f64, y:f64){
        self.pen_up();
        self.go_to([x, y]);
        self.pen_down();
        self.forward(1.0);
        self.home();
    }
}


fn main() {
    let mut turtle = Turtle::new();
    turtle.set_speed(25);
    for x in -100..100 {
        let y = (x as f64)/10.0;
        turtle.point(x as f64, y.sin()*10.0);
    }
}

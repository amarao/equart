use criterion::{black_box, criterion_group, criterion_main, Criterion};
use equart::quadtree::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fill 500 x 500", |b| b.iter(
        ||{

            let mut q = QuadTree::new(Boundry::from_coords(-300.0, -300.0, 300.0, 300.0));
            for x in -250..250{
                for y in -250..250{
                    let p = Point::new(x as f64, y as f64);
                    q.append_point(p, (x,y)).unwrap();
                }
            }
            q.search(Point::new(0.0, 0.0)).is_some();
        }
    ));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use equart::quadtree::*;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("quadtree");
    group.measurement_time(core::time::Duration::new(10, 0));
    group.bench_function("fill_20x20", |b| b.iter(
        ||{
            let mut q = QuadTree::new(Boundry::from_coords(0.0, 0.0, 20.0, 20.0));
            for x in 0..20{
                for y in 0..20{
                    let p = Point::new(black_box(x as f64), black_box(y as f64));
                    black_box(q.append_point(p, black_box((x,y))));
                }
            }
        }
    ));
    let mut q = QuadTree::new(Boundry::from_coords(0.0, 0.0, 20.0, 20.0));
    for x in 0..20{
        for y in 0..20{
            let p = Point::new(black_box(x as f64), black_box(y as f64));
            black_box(q.append_point(p, black_box((x,y))));
        }
    }
    group.bench_function("search_positive_20x20", |b| b.iter(
        
        
        ||{
            
            for x in 0..20{
                for y in 0..20{
                    black_box(q.search(black_box(Point::new(x as f64, y as f64))));
                }
            }
        }
    ));
    group.bench_function("search_negative_20x20", |b| b.iter(
        ||{
            for x in 0..20{
                for y in 0..20{
                    black_box(q.search(Point::new(0.9999 * x as f64, 0.9999 *y as f64)));
                }
            }
        }
    ));
}


criterion_group!(bench1, bench);
criterion_main!(bench1);
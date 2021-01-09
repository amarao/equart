use criterion::{black_box, criterion_group, criterion_main, Criterion};
use equart::quadtree::*;

pub fn bench(c: &mut Criterion) {
    let mut q = QuadTree::new(Boundry::from_coords(0.0, 0.0, 20.0, 20.0));
    let mut group = c.benchmark_group("quadtree");
    group.measurement_time(core::time::Duration::new(10, 0));
    group.bench_function("fill_20x20", |b| b.iter(
        ||{
            for x in 0..20{
                for y in 0..20{
                    let p = Point::new(black_box(x as f64), black_box(y as f64));
                    black_box(q.append_point(p, black_box(x+y)));
                }
            }
        }
    ));
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

pub fn better(c: &mut Criterion) {
    let mut group = c.benchmark_group("better");
    group.bench_function("Box 100", |b| b.iter(
        ||{
        let mut v = Vec::new();
        for y in 0..100{
            
                let b = Box::new(black_box(42.0f64));
                v.push(b);
            }
        }
    ));
    group.bench_function("array", |b| b.iter(
        ||{
        let mut v = Vec::new();
        let mut z:f64 = 0.0f64;
        for y in 0..100{
            for x in 0..y{
                if v[x] == black_box(v[x]){
                    z = x as f64 + 1.0f64;
                }
            }
            v.push(z);
            }
        }
    ));
}


criterion_group!(bench1, bench);
criterion_group!(bench2, better);
criterion_main!(bench1, bench2);
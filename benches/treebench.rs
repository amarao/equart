use criterion::{black_box, criterion_group, criterion_main, Criterion};
use equart::quadtree::*;

pub fn fill_deep(c: &mut Criterion) {
    c.bench_function("fill_deep", |b| b.iter(
        ||{
            let mut q = QuadTree::new(Boundry::from_coords(0.0, 0.0, 20.0, 20.0));
            for x in 1..201{
                for y in 1..201{
                    let p = Point::new(black_box(1.0/x as f64), black_box(1.0/y as f64));
                    black_box(q.append_point(p, black_box((x,y))));
                }
            }
        }
    ));
}

pub fn fill_wide(c: &mut Criterion) {
    c.bench_function("fill_wide", |b| b.iter(
        ||{
            let mut q = QuadTree::new(Boundry::from_coords(0.0, 0.0, 200.0, 200.0));
            for x in 0..200{
                for y in 0..200{
                    let p = Point::new(black_box(x as f64), black_box(y as f64));
                    black_box(q.append_point(p, black_box((x,y))));
                }
            }
        }
    ));
}

pub fn search_deep(c: &mut Criterion) {
    let mut q = QuadTree::new(Boundry::from_coords(0.0, 0.0, 200.0, 200.0));
    for x in 1..201{
        for y in 1..201{
            let p = Point::new(black_box(1.0/x as f64), black_box(1.0/y as f64));
            black_box(q.append_point(p, black_box((x,y))));
        }
    }

    c.bench_function("search_deep", |b| b.iter(
        ||{
            q.values_in_area(Boundry::from_coords(0.0, 0.0, 1.0, 1.0));
        }
    ));
}

pub fn search_wide(c: &mut Criterion) {
    let mut q = QuadTree::new(Boundry::from_coords(0.0, 0.0, 200.0, 200.0));
    for x in 0..200{
        for y in 0..200{
            let p = Point::new(black_box(x as f64), black_box(y as f64));
            black_box(q.append_point(p, black_box((x,y))));
        }
    }
    c.bench_function("search wide", |b| b.iter(
        ||{
            q.values_in_area(Boundry::from_coords(50.0, 50.0, 150.0, 150.0));
        }
    ));
}

criterion_group!(bench_fill, fill_deep, fill_wide);
criterion_group!(bench_search, search_deep, search_wide);
criterion_main!(bench_fill, bench_search);
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use la_texer::Parser;

/// Benchmarks the performance of parsing a complex LaTeX expression.
fn bench_parse_complex(c: &mut Criterion) {
    let input = "\\alpha x ".repeat(10000);

    c.bench_function("parse_complex", |b| {
        b.iter(|| {
            let parser = Parser::new(black_box(&input));
            black_box(parser.parse())
        })
    });
}

/// Benchmarks the performance of parsing a simple LaTeX expression.
fn bench_parse_simple(c: &mut Criterion) {
    let input = r#"\frac{\frac{\frac{100000}{x_{7y-2}}}{x + 3^{24}}}{y - 2}"#.repeat(1000);

    c.bench_function("parse_simple", |b| {
        b.iter(|| {
            let parser = Parser::new(black_box(&input));
            black_box(parser.parse())
        })
    });
}

criterion_group!(benches, bench_parse_complex, bench_parse_simple,);
criterion_main!(benches);

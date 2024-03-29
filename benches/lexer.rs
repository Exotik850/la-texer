use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Benchmarks the performance of tokenizing a complex LaTeX expression.
fn bench_tokenize_complex(c: &mut Criterion) {
    let input = r#"
  \frac{\dv}{\dv x}\int_{a(x)}^{b(x)}f(x,t)\dv t = f(x,b(x))\cdot \frac{\dv}{\dv x} b(x) - f(x, a(x))\cdot \frac{\dv}{\dv x}a(x) + \int_{a(x)}^{b(x)}\frac{\partial}{\partial x}f(x,t)\dv t
  "#;

    c.bench_function("tokenize_complex", |b| {
        b.iter(|| {
            let lexer = la_texer::Lexer::new(black_box(input));
            for token in lexer {
                black_box(token);
            }
        })
    });
}

/// Benchmarks the performance of tokenizing a simple LaTeX expression.
fn bench_tokenize_simple(c: &mut Criterion) {
    let input = r#"\frac{x + 1}{y - 2}"#;

    c.bench_function("tokenize_simple", |b| {
        b.iter(|| {
            let lexer = la_texer::Lexer::new(black_box(input));
            for token in lexer {
                black_box(token);
            }
        })
    });
}

criterion_group!(benches, bench_tokenize_complex, bench_tokenize_simple,);
criterion_main!(benches);

use la_texer::{Node, ParseNodes};

fn main() {
    let input = r#"\frac{1}{2} \displaystyle\sum_{i}{(y^{(i)} - \theta^{T}x^{(i)})^{2}}"#;
    let start = std::time::Instant::now();
    let ast = input.parse_latex();
    let elapsed = start.elapsed();
    println!("{ast:#?} \n Done in {elapsed:?}");
}

use la_texer::Parser;

fn main() {
  let input = r#"\frac{1}{2} \displaystyle\sum_{i}{(y^{(i)} - \theta^{T}x^{(i)})^{2}}"#;
  let start = std::time::Instant::now();
  let ast = Parser::new(input).parse();
  println!("{:#?} \n Done in {:?}", ast, start.elapsed());
}
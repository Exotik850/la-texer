use la_texer::Parser;

fn main() {
  let mut input = String::new();
  loop {
    std::io::stdin().read_line(&mut input).unwrap();
    let start = std::time::Instant::now();
    let ast = Parser::new(&input).parse();
    let elapsed = start.elapsed();
    println!("{:#?} \n Done in {:?}", ast, elapsed);
    input.clear();
  }

}
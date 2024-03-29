use la_texer::ParseNodes;

fn main() {
    let input = r#"\text{Hello World}"#;
    let start = std::time::Instant::now();
    let ast = input.parse_latex();
    let elapsed = start.elapsed();
    println!("{ast:#?} \n Done in {elapsed:?}");
}

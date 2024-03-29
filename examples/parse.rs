use la_texer::IntoTexNodes;

fn main() {
    let input = r#"\text{Hello World}"#;
    let start = std::time::Instant::now();
    let ast = input.into_nodes();
    let elapsed = start.elapsed();
    println!("{ast:#?} \n Done in {elapsed:?}");
}

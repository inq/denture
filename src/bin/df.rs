use std::env;

fn process(filename: &str) -> Result<(), failure::Error> {
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let tokenizer = denture::parser::Tokenizer::from_reader(reader)?;

    println!("{:?}", tokenizer);

    Ok(())
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("usage: df file");
    } else {
        process(&args[1]).unwrap();
    }
}

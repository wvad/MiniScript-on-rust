mod lexer;

fn main() {
    let filename = "test.msct";
    for a in lexer::parse(&std::fs::read_to_string(filename).expect("Failed to read file"), filename) {
        println!("{:?}", a);
    }
}

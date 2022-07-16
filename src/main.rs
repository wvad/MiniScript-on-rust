mod lexer;
mod parser;

fn main() {
    let input = std::fs::read_to_string("test.msct").expect("Failed to read file");
    let mut tokens = match lexer::parse(&input) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Failed: {:?}", e);
            return;
        }
    };
    match parser::parse_expression(&mut tokens) {
        Ok(expr) => println!("Parsed: {:?}", expr),
        Err(e) => eprintln!("Failed: {:?}", e),
    }
}

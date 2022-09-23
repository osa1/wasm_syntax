use std::process::exit;

use wasm_syntax_expanded::{Decode, Encode, Module};

fn main() {
    let file = std::env::args().nth(1).unwrap();
    let file_contents = std::fs::read(&file).unwrap();
    let (module, rest) = Module::decode(&file_contents).unwrap();

    if !rest.is_empty() {
        println!("Buffer is not empty after parsing module: {:?}", rest);
        exit(1);
    }

    println!("{:?}", module);

    let mut encoded = Vec::with_capacity(file_contents.len());
    module.encode(&mut encoded);

    let (encoded_parsed, rest) = Module::decode(&encoded).unwrap();

    if !rest.is_empty() {
        println!(
            "Buffer is not empty after parsing generated module: {:?}",
            rest
        );
        exit(1);
    }

    if encoded_parsed != module {
        println!("Coded module is not the same as the original decoded module");
        exit(1);
    }
}

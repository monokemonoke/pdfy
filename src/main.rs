use crate::reader::PdfReader;

mod reader;

fn main() {
    let pdf = PdfReader::new("202006tabataiga.pdf").unwrap();

    dbg!(pdf);

    println!("Hi");
}

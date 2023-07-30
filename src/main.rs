use crate::reader::PdfReader;

mod reader;
mod utils;

fn main() {
    let file = std::env::args().nth(1).expect("file name is required");

    let pdf = PdfReader::new(&file).unwrap();

    dbg!(&pdf);

    pdf.for_test();

    println!("Hi");
}

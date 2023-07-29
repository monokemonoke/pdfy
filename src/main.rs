use crate::reader::PdfReader;

mod reader;
mod utils;

fn main() {
    let pdf = PdfReader::new("202006tabataiga.pdf").unwrap();

    dbg!(&pdf);

    pdf.for_test();

    println!("Hi");
}

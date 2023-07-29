use std::fs::File;

#[derive(Debug)]
pub struct PdfReader {
    _file: File,
}

impl PdfReader {
    pub fn new(path: &str) -> Result<Self, String> {
        let file = match File::open(path) {
            Err(_) => return Err("ファイルを読み込めませんでした".to_string()),
            Ok(v) => v,
        };

        Ok(Self { _file: file })
    }
}

use std::{
    fs::File,
    io::{BufReader, Seek, SeekFrom},
};

use crate::utils::read_previous_line;

#[derive(Debug)]
pub struct PdfReader {
    _file: File,
}

const CHECK_EOF_LIMIT: usize = 16;

impl PdfReader {
    pub fn new(path: &str) -> Result<Self, &str> {
        let file = match File::open(path) {
            Err(_) => return Err("ファイルを読み込めませんでした"),
            Ok(v) => v,
        };

        Ok(Self { _file: file })
    }

    #[allow(unused_must_use)]
    pub fn for_test(&self) {
        dbg!(self.check_eof_with_limit());
    }

    fn check_eof_with_limit(&self) -> Result<u64, &str> {
        let mut reader = BufReader::new(&self._file);

        reader.seek(SeekFrom::End(-1)).unwrap();

        for _ in 0..CHECK_EOF_LIMIT {
            let line = read_previous_line(&mut reader).unwrap();
            if line.starts_with("%%EOF") {
                return Ok(reader.stream_position().unwrap());
            }
        }

        Err("EOFが見つかりませんでした")
    }
}

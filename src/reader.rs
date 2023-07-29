use std::{
    fs::File,
    io::{BufReader, Seek, SeekFrom},
};

use crate::utils;

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
        let eof_pos = self.check_eof_with_limit().unwrap();
        dbg!(&eof_pos);

        let xref_pos = self.parse_xref_table_pos(eof_pos).unwrap();
        dbg!(&xref_pos);
    }

    fn check_eof_with_limit(&self) -> Result<u64, &str> {
        let mut reader = BufReader::new(&self._file);
        reader.seek(SeekFrom::End(-1)).unwrap();

        for _ in 0..CHECK_EOF_LIMIT {
            let line = utils::read_previous_line(&mut reader).unwrap();
            if line.starts_with("%%EOF") {
                return Ok(reader.stream_position().unwrap());
            }
        }

        Err("EOFが見つかりませんでした")
    }

    fn parse_xref_table_pos(&self, eof_pos: u64) -> Result<u64, &str> {
        let mut reader = BufReader::new(&self._file);
        reader.seek(SeekFrom::Start(eof_pos)).unwrap();

        let xref_byte = utils::read_previous_line(&mut reader).unwrap();
        match xref_byte.parse::<u64>() {
            Err(_) => Err("xref tableの場所がパースできませんでした"),
            Ok(n) => Ok(n),
        }
    }
}

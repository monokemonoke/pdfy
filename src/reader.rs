use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
};

use crate::utils;

#[derive(Debug)]
pub struct PdfReader {
    _file: File,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct XrefRecord {
    byte: u64,
    generation: u64,
    obj_type: ObjType,
}

#[derive(Debug)]
pub enum ObjType {
    F,
    N,
}

impl ObjType {
    pub fn new(str: &str) -> Result<Self, ()> {
        match str {
            "f" => Ok(Self::F),
            "n" => Ok(Self::N),
            _ => Err(()),
        }
    }
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

        let table = self.parse_xref_table(xref_pos).unwrap();
        dbg!(&table);
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

    fn parse_xref_table(&self, xref_pos: u64) -> Result<Vec<XrefRecord>, &str> {
        let mut reader = BufReader::new(&self._file);
        reader.seek(SeekFrom::Start(xref_pos)).unwrap();

        let mut buf = [0; 4];
        reader.read(&mut buf).unwrap();
        if !buf.starts_with(b"xref") {
            reader.seek(SeekFrom::Current(-4)).unwrap();
        }

        let mut buf = [0; 1];
        loop {
            reader.read(&mut buf).unwrap();
            if &buf != b"\n" && &buf != b"\r" {
                reader.seek(SeekFrom::Current(-1)).unwrap();
                break;
            }
        }

        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        let buf = buf.trim().split(' ').nth(1);
        let buf = if let Some(s) = buf {
            s
        } else {
            return Err("オブジェクトの総数を取得できませんでした");
        };
        let objects_length: u64 = if let Ok(n) = buf.parse() {
            n
        } else {
            return Err("オブジェクトの総数を取得できませんでした");
        };

        dbg!(&objects_length);

        let mut table: Vec<XrefRecord> = Vec::new();
        for _ in 0..objects_length {
            let mut buf = String::new();
            reader.read_line(&mut buf).unwrap();

            let row: Vec<&str> = buf.split_whitespace().collect();
            let (byte, gen, obj_type) = match row[..] {
                [byte, gen, obj_type] => (byte, gen, obj_type),
                _ => return Err("オブジェクトの総数を取得できませんでした"),
            };
            let byte: u64 = match byte.parse() {
                Ok(n) => n,
                Err(_) => return Err("オブジェクトの総数を取得できませんでした"),
            };
            let generation: u64 = match gen.parse() {
                Ok(n) => n,
                Err(_) => return Err("オブジェクトの総数を取得できませんでした"),
            };
            let obj_type: ObjType = match ObjType::new(obj_type) {
                Ok(t) => t,
                Err(_) => return Err("オブジェクトの総数を取得できませんでした"),
            };

            table.push(XrefRecord {
                byte,
                generation,
                obj_type,
            })
        }

        Ok(table)
    }
}

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

        let _table = self.parse_xref_table(xref_pos).unwrap();
        // dbg!(&table);

        let trailer_pos = self.get_tailer_obj_position().unwrap();
        dbg!(&trailer_pos);
    }

    fn check_eof_with_limit(&self) -> Result<u64, &str> {
        let mut reader = BufReader::new(&self._file);
        reader.seek(SeekFrom::End(-1)).or(Err("IOエラー"))?;

        for _ in 0..CHECK_EOF_LIMIT {
            let line = utils::read_previous_line(&mut reader).or(Err("IOエラー"))?;
            if line.starts_with("%%EOF") {
                return reader.stream_position().or(Err("IOエラー"));
            }
        }

        Err("EOFが見つかりませんでした")
    }

    fn parse_xref_table_pos(&self, eof_pos: u64) -> Result<u64, &str> {
        let mut reader = BufReader::new(&self._file);
        reader.seek(SeekFrom::Start(eof_pos)).or(Err("IOエラー"))?;

        utils::read_previous_line(&mut reader)
            .or(Err("IOエラー"))?
            .parse::<u64>()
            .or(Err("xref tableの場所がパースできませんでした"))
    }

    fn parse_xref_table(&self, xref_pos: u64) -> Result<Vec<XrefRecord>, &str> {
        let mut reader = BufReader::new(&self._file);
        reader.seek(SeekFrom::Start(xref_pos)).or(Err("IOエラー"))?;

        let mut buf = [0; 4];
        reader.read(&mut buf).or(Err("IOエラー"))?;
        if !buf.starts_with(b"xref") {
            reader.seek(SeekFrom::Current(-4)).or(Err("IOエラー"))?;
        }

        let mut buf = [0; 1];
        loop {
            reader.read(&mut buf).or(Err("IOエラー"))?;
            if &buf != b"\n" && &buf != b"\r" {
                reader.seek(SeekFrom::Current(-1)).or(Err("IOエラー"))?;
                break;
            }
        }

        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        let objects_length = buf
            .trim()
            .split(' ')
            .nth(1)
            .ok_or("cannot parse object's lengths")?
            .parse::<u64>()
            .or(Err("cannot parse object's lengths"))?;

        let mut table: Vec<XrefRecord> = Vec::new();
        for _ in 0..objects_length {
            let mut buf = String::new();
            reader.read_line(&mut buf).or(Err("IOエラー"))?;

            let row: Vec<&str> = buf.split_whitespace().collect();
            let (byte, gen, obj_type) = match row[..] {
                [byte, gen, obj_type] => (byte, gen, obj_type),
                _ => return Err("cannot parse object's lengths"),
            };
            let byte: u64 = byte.parse().or(Err("cannot parse obj info"))?;
            let generation: u64 = gen.parse().or(Err("cannot parse obj info"))?;
            let obj_type = ObjType::new(obj_type).or(Err("cannot parse obj info"))?;

            table.push(XrefRecord {
                byte,
                generation,
                obj_type,
            })
        }

        Ok(table)
    }

    fn get_tailer_obj_position(&self) -> Result<u64, &str> {
        let mut reader = BufReader::new(&self._file);
        reader.seek(SeekFrom::End(-1)).or(Err("IOエラー"))?;

        for _ in 0..CHECK_EOF_LIMIT {
            let line = utils::read_previous_line(&mut reader).or(Err("IOエラー"))?;

            if line.starts_with("trailer") {
                // objectが始まる位置まで読み飛ばす
                let mut buf = [0; 1];
                while reader
                    .read(&mut buf)
                    .ok()
                    .filter(|_| &buf == b"<")
                    .is_none()
                {}
                reader.seek(SeekFrom::Current(-1)).or(Err("IOエラー"))?;

                return reader.stream_position().or(Err("IOエラー"));
            }
        }

        Err("trailerが見つかりませんでした")
    }
}

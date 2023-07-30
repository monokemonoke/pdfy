use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    str::from_utf8,
};

use crate::utils;

#[derive(Debug)]
pub struct PdfReader {
    _file: File,
}

#[derive(Hash, PartialEq, Eq, Debug)]
struct XrefTableKey {
    id: u64,
    generation: u64,
}

type XrefTable = HashMap<XrefTableKey, XrefRecord>;

#[derive(Debug)]
#[allow(dead_code)]
pub struct XrefRecord {
    byte: u64,
    obj_type: ObjType,
}

#[derive(Debug)]
pub enum ObjType {
    F,
    N,
}

#[derive(Debug)]
#[allow(dead_code)]
struct ObjectRef {
    id: u64,
    generation: u64,
}

#[derive(Debug)]
#[allow(dead_code)]
struct TrailerObjct {
    size: u64,
    info: ObjectRef,
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
        dbg!(&_table);

        let trailer_pos = self.get_tailer_obj_position().unwrap();
        dbg!(&trailer_pos);

        let trailer_obj = self.parse_trailer_obj(trailer_pos).unwrap();
        dbg!(&trailer_obj);
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

    fn parse_xref_table(&self, xref_pos: u64) -> Result<XrefTable, &str> {
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

        let mut table = XrefTable::new();
        for id in 0..objects_length {
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

            let key = XrefTableKey { id, generation };
            let record = XrefRecord { byte, obj_type };
            table.insert(key, record);
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

    fn parse_trailer_obj(&self, obj_pos: u64) -> Result<TrailerObjct, &'static str> {
        let mut reader = BufReader::new(&self._file);
        reader.seek(SeekFrom::Start(obj_pos)).or(Err("IOエラー"))?;

        let mut obj = String::new();
        loop {
            let mut buf = [0; 1];
            reader.read_exact(&mut buf).or(Err("IOエラー"))?;

            match &buf {
                b">" => {
                    obj.push_str(">");
                    reader.read_exact(&mut buf).or(Err("IOエラー"))?;
                    match &buf {
                        b">" => {
                            obj.push_str(">");
                            break;
                        }
                        s => obj.push_str(from_utf8(s).or(Err("IOエラー"))?),
                    }
                }
                s => obj.push_str(from_utf8(s).or(Err("IOエラー"))?),
            };
        }

        use regex::Regex;

        let size_res = Regex::new(r#"/Size (?<num>\d+)"#)
            .or(Err("size regex is not valid"))?
            .captures(&obj)
            .ok_or("size param is not found")?;

        let info_res = Regex::new(r#"/Info (?<id>\d+) (?<gen>\d+)"#)
            .or(Err("info regex is not valid"))?
            .captures(&obj)
            .ok_or("info param is not found")?;

        let size: u64 = size_res["num"].parse().or(Err("size should be digit"))?;
        let id: u64 = (&info_res["id"]).parse().or(Err("id should be digit"))?;
        let generation: u64 = (&info_res["gen"]).parse().or(Err("gen should be digit"))?;

        Ok(TrailerObjct {
            size,
            info: ObjectRef { id, generation },
        })
    }
}

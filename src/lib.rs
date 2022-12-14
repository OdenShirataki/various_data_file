use file_mmap::FileMmap;
use std::{io, path::Path};

mod flagment;

#[derive(Clone, Default, PartialEq)]
pub struct DataAddress {
    offset: i64,
    len: u64,
}
impl DataAddress {
    pub fn offset(&self) -> i64 {
        self.offset
    }
    pub fn len(&self) -> u64 {
        self.len
    }
}
pub struct Data<'a> {
    address: DataAddress,
    data: &'a VariousDataFile,
}
impl Data<'_> {
    pub fn bytes(&self) -> &[u8] {
        unsafe { self.data.bytes(&self.address) }
    }
    pub fn address(&self) -> &DataAddress {
        &self.address
    }
}

pub struct VariousDataFile {
    filemmap: FileMmap,
    fragment: flagment::Fragment,
}
impl VariousDataFile {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let mut filemmap = FileMmap::new(path)?;
        if filemmap.len()? == 0 {
            filemmap.set_len(1)?;
        }
        Ok(VariousDataFile {
            filemmap,
            fragment: {
                let mut path = path.to_path_buf();
                path.set_file_name(
                    &(if let Some(file_name) = path.file_name() {
                        file_name.to_string_lossy()
                    } else {
                        "".into()
                    }
                    .into_owned()
                        + ".f"),
                );
                flagment::Fragment::new(path)?
            },
        })
    }
    pub unsafe fn bytes(&self, word: &DataAddress) -> &[u8] {
        self.filemmap
            .bytes(word.offset() as isize, word.len as usize)
    }
    pub unsafe fn offset(&self, addr: isize) -> *const u8 {
        self.filemmap.offset(addr)
    }
    pub fn insert(&mut self, target: &[u8]) -> io::Result<Data> {
        let len = target.len();
        match self.fragment.search_blank(len) {
            Some(r) => {
                self.filemmap.write(r.string_addr as isize, target)?;
                unsafe {
                    self.fragment.release(r.fragment_id, len);
                }
                Ok(Data {
                    address: DataAddress {
                        offset: r.string_addr as i64,
                        len: len as u64,
                    },
                    data: self,
                })
            }
            None => {
                let addr = self.filemmap.append(target)?;
                Ok(Data {
                    address: DataAddress {
                        offset: addr as i64,
                        len: len as u64,
                    },
                    data: self,
                })
            }
        }
    }
    pub fn remove(&mut self, ystr: &DataAddress) -> io::Result<()> {
        self.filemmap
            .write_0(ystr.offset as isize, ystr.len as usize)?;
        self.fragment.insert(ystr)?;
        Ok(())
    }
}

#[test]
fn test() {
    let dir = "./vdf-test";
    if std::path::Path::new(dir).exists() {
        std::fs::remove_dir_all(dir).unwrap();
        std::fs::create_dir_all(dir).unwrap();
    } else {
        std::fs::create_dir_all(dir).unwrap();
    }
    if let Ok(mut s) = VariousDataFile::new(&(dir.to_owned() + "/test.str")) {
        let noah = s.insert(b"Noah").unwrap().address;
        let liam = s.insert(b"Liam").unwrap().address;
        let olivia = s.insert(b"Olivia").unwrap().address;

        s.remove(&noah).unwrap();

        if let Ok(w) = s.insert(b"Renamed Noah") {
            assert_eq!(
                "Renamed Noah".to_string(),
                std::str::from_utf8(w.bytes()).unwrap().to_string()
            );
        }
        s.remove(&liam).unwrap();
        if let Ok(w) = s.insert(b"Renamed Liam") {
            assert_eq!(
                "Renamed Liam".to_string(),
                std::str::from_utf8(w.bytes()).unwrap().to_string()
            );
        }
        s.remove(&olivia).unwrap();
        if let Ok(w) = s.insert(b"Renamed Olivia") {
            assert_eq!(
                "Renamed Olivia".to_string(),
                std::str::from_utf8(w.bytes()).unwrap().to_string()
            );
        }
        if let Ok(w) = s.insert(b"Noah") {
            assert_eq!(
                "Noah".to_string(),
                std::str::from_utf8(w.bytes()).unwrap().to_string()
            );
        }
        if let Ok(w) = s.insert(b"Liam") {
            assert_eq!(
                "Liam".to_string(),
                std::str::from_utf8(w.bytes()).unwrap().to_string()
            );
        }
        if let Ok(w) = s.insert(b"Olivia") {
            assert_eq!(
                "Olivia".to_string(),
                std::str::from_utf8(w.bytes()).unwrap().to_string()
            );
        }
    }
}

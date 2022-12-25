use file_mmap::FileMmap;
use std::mem::ManuallyDrop;

use crate::DataAddress;

pub(super) struct FragmentGetResult {
    pub(super) fragment_id: u64,
    pub(super) string_addr: u64,
}
pub(super) struct Fragment {
    filemmap: FileMmap,
    list: ManuallyDrop<Box<DataAddress>>,
    record_count: ManuallyDrop<Box<u64>>,
}
const DATAADDRESS_SIZE: usize = std::mem::size_of::<DataAddress>();
const COUNTER_SIZE: usize = std::mem::size_of::<u64>();
const INIT_SIZE: usize = COUNTER_SIZE + DATAADDRESS_SIZE;
impl Fragment {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let filemmap = FileMmap::new(path, INIT_SIZE as u64)?;
        let list = unsafe { filemmap.offset(COUNTER_SIZE as isize) } as *mut DataAddress;
        let counter = filemmap.as_ptr() as *mut u64;
        Ok(Fragment {
            filemmap,
            list: ManuallyDrop::new(unsafe { Box::from_raw(list) }),
            record_count: ManuallyDrop::new(unsafe { Box::from_raw(counter) }),
        })
    }
    pub fn insert(&mut self, ystr: &DataAddress) -> Result<u64, std::io::Error> {
        **self.record_count += 1;
        let size = INIT_SIZE + DATAADDRESS_SIZE * **self.record_count as usize;
        if self.filemmap.len() < size as u64 {
            self.filemmap.set_len(size as u64)?;
        }
        unsafe {
            *(&mut **self.list as *mut DataAddress).offset(**self.record_count as isize) =
                ystr.clone();
        }
        Ok(**self.record_count)
    }
    pub unsafe fn release(&mut self, row: u64, len: u64) {
        let mut s = &mut *(&mut **self.list as *mut DataAddress).offset(row as isize);
        s.offset += len as i64;
        s.len -= len;

        if s.len == 0 && row == **self.record_count {
            **self.record_count -= 1;
        }
    }
    pub fn search_blank(&self, len: u64) -> Option<FragmentGetResult> {
        if **self.record_count == 0 {
            None
        } else {
            for i in -(**self.record_count as i64)..0 {
                let index = (-i) as u64;
                let s = unsafe { &*(&**self.list as *const DataAddress).offset(index as isize) };
                if s.len >= len {
                    return Some(FragmentGetResult {
                        fragment_id: index,
                        string_addr: s.offset as u64,
                    });
                }
            }
            None
        }
    }
}

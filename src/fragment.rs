use std::{mem::size_of, num::NonZeroU64, path::Path};

use file_mmap::FileMmap;

use crate::DataAddress;

pub(super) struct FragmentGetResult {
    pub(super) fragment_id: NonZeroU64,
    pub(super) addr: u64,
}

pub(super) struct Fragment {
    filemmap: FileMmap,
}

const DATAADDRESS_SIZE: u64 = size_of::<DataAddress>() as u64;
const COUNTER_SIZE: u64 = size_of::<u64>() as u64;
const INIT_SIZE: u64 = COUNTER_SIZE + DATAADDRESS_SIZE;

impl Fragment {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut filemmap = FileMmap::new(path).unwrap();
        if filemmap.is_empty() {
            filemmap.set_len(INIT_SIZE).unwrap();
        }
        Self { filemmap }
    }

    unsafe fn list(&self) -> *const DataAddress {
        unsafe { self.filemmap.as_ptr().offset(COUNTER_SIZE as isize) as *const DataAddress }
    }

    unsafe fn list_mut(&mut self) -> *mut DataAddress {
        unsafe { self.filemmap.as_ptr().offset(COUNTER_SIZE as isize) as *mut DataAddress }
    }

    pub fn insert(&mut self, addr: DataAddress) {
        let record_count = self.filemmap.as_ptr() as *mut u64;
        let record_count = unsafe {
            *record_count += 1;
            *record_count
        };
        let size = INIT_SIZE + DATAADDRESS_SIZE * record_count;
        if self.filemmap.len() < size {
            self.filemmap.set_len(size).unwrap();
        }
        unsafe {
            *self.list_mut().offset(record_count as isize) = addr;
        }
    }

    pub unsafe fn release(&mut self, row: NonZeroU64, len: usize) {
        unsafe {
            let row = row.get();
            let s = &mut *self.list_mut().offset(row as isize);
            s.offset += len as i64;
            s.len -= len as u64;

            let record_count = self.filemmap.as_ptr() as *mut u64;
            if s.len == 0 && row == *record_count {
                *record_count -= 1;
            }
        }
    }

    pub fn search_blank(&self, len: usize) -> Option<FragmentGetResult> {
        let record_count = unsafe { *(self.filemmap.as_ptr() as *const u64) };
        if record_count != 0 {
            for index in (-(record_count as isize)..0).map(|i| -i) {
                let s = unsafe { &*self.list().offset(index) };
                if s.len as usize >= len {
                    return Some(FragmentGetResult {
                        fragment_id: unsafe { NonZeroU64::new_unchecked(index as u64) },
                        addr: s.offset as u64,
                    });
                }
            }
        }
        None
    }
}

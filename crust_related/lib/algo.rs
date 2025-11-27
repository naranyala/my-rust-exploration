// crust_algorithms/lib.rs
#![no_std]

pub mod sort {
    pub fn quicksort<T: Ord>(slice: &mut [T]) {
        if slice.len() <= 1 {
            return;
        }
        
        let pivot_index = partition(slice);
        quicksort(&mut slice[..pivot_index]);
        quicksort(&mut slice[pivot_index + 1..]);
    }

    fn partition<T: Ord>(slice: &mut [T]) -> usize {
        let pivot_index = slice.len() / 2;
        slice.swap(pivot_index, slice.len() - 1);
        
        let mut i = 0;
        for j in 0..slice.len() - 1 {
            if slice[j] <= slice[slice.len() - 1] {
                slice.swap(i, j);
                i += 1;
            }
        }
        
        slice.swap(i, slice.len() - 1);
        i
    }

    pub fn binary_search<T: Ord>(slice: &[T], target: &T) -> Option<usize> {
        let mut left = 0;
        let mut right = slice.len();
        
        while left < right {
            let mid = left + (right - left) / 2;
            match slice[mid].cmp(target) {
                core::cmp::Ordering::Equal => return Some(mid),
                core::cmp::Ordering::Less => left = mid + 1,
                core::cmp::Ordering::Greater => right = mid,
            }
        }
        
        None
    }
}

pub mod crypto {
    pub fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFFFFFFu32;
        
        for &byte in data {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }
        
        !crc
    }

    pub fn simple_hash(data: &[u8]) -> u64 {
        let mut hash: u64 = 14695981039346656037;
        for &byte in data {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        hash
    }
}

pub mod compression {
    pub fn run_length_encode(data: &[u8]) -> Vec<u8, 1024> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < data.len() {
            let byte = data[i];
            let mut count = 1;
            
            while i + count < data.len() && data[i + count] == byte && count < 255 {
                count += 1;
            }
            
            let _ = result.push(count as u8);
            let _ = result.push(byte);
            i += count;
        }
        
        result
    }

    pub fn run_length_decode(data: &[u8]) -> Result<Vec<u8, 1024>, &'static str> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i + 1 < data.len() {
            let count = data[i] as usize;
            let byte = data[i + 1];
            
            for _ in 0..count {
                result.push(byte)?;
            }
            
            i += 2;
        }
        
        Ok(result)
    }
}

// collections.rs
#![no_std]

pub mod collections {
    pub struct Vec<T, const CAP: usize> {
        data: [Option<T>; CAP],
        len: usize,
    }
    
    impl<T, const CAP: usize> Vec<T, CAP> {
        pub const fn new() -> Self {
            Self {
                data: [const { None }; CAP],
                len: 0,
            }
        }
        
        pub fn push(&mut self, item: T) -> Result<(), &'static str> {
            if self.len >= CAP {
                return Err("Vector full");
            }
            self.data[self.len] = Some(item);
            self.len += 1;
            Ok(())
        }
        
        pub fn pop(&mut self) -> Option<T> {
            if self.len == 0 {
                return None;
            }
            self.len -= 1;
            self.data[self.len].take()
        }
        
        pub fn get(&self, index: usize) -> Option<&T> {
            if index < self.len {
                self.data[index].as_ref()
            } else {
                None
            }
        }
        
        pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
            if index < self.len {
                self.data[index].as_mut()
            } else {
                None
            }
        }
        
        pub fn len(&self) -> usize {
            self.len
        }
        
        pub fn is_empty(&self) -> bool {
            self.len == 0
        }
        
        pub fn capacity(&self) -> usize {
            CAP
        }
    }
    
    impl<T: Copy, const CAP: usize> Vec<T, CAP> {
        pub fn to_array(&self) -> [Option<T>; CAP] {
            self.data
        }
    }
    
    // A simple hash map for small key sets
    pub struct HashMap<K, V, const CAP: usize> 
    where
        K: Eq + Copy,
        V: Copy,
    {
        entries: [Option<(K, V)>; CAP],
        len: usize,
    }
    
    impl<K, V, const CAP: usize> HashMap<K, V, CAP> 
    where
        K: Eq + Copy,
        V: Copy,
    {
        pub const fn new() -> Self {
            Self {
                entries: [const { None }; CAP],
                len: 0,
            }
        }
        
        pub fn insert(&mut self, key: K, value: V) -> Result<(), &'static str> {
            if self.len >= CAP {
                return Err("HashMap full");
            }
            
            // Simple linear probing
            for entry in &mut self.entries {
                match entry {
                    Some((k, _)) if *k == key => {
                        *entry = Some((key, value));
                        return Ok(());
                    }
                    None => {
                        *entry = Some((key, value));
                        self.len += 1;
                        return Ok(());
                    }
                    _ => {}
                }
            }
            
            Err("No space available")
        }
        
        pub fn get(&self, key: K) -> Option<&V> {
            for entry in &self.entries {
                if let Some((k, v)) = entry {
                    if *k == key {
                        return Some(v);
                    }
                }
            }
            None
        }
        
        pub fn remove(&mut self, key: K) -> Option<V> {
            for entry in &mut self.entries {
                if let Some((k, v)) = entry {
                    if *k == key {
                        self.len -= 1;
                        return entry.take().map(|(_, v)| v);
                    }
                }
            }
            None
        }
        
        pub fn len(&self) -> usize {
            self.len
        }
    }
}

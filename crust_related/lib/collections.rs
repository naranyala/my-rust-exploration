// crust_collections/lib.rs
#![no_std]

pub mod vec {
    pub struct Vec<T, const CAP: usize> {
        data: [core::mem::MaybeUninit<T>; CAP],
        len: usize,
    }

    impl<T, const CAP: usize> Vec<T, CAP> {
        pub const fn new() -> Self {
            Self {
                data: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
                len: 0,
            }
        }

        pub fn push(&mut self, value: T) -> Result<(), &'static str> {
            if self.len >= CAP {
                return Err("Capacity exceeded");
            }
            self.data[self.len].write(value);
            self.len += 1;
            Ok(())
        }

        pub fn pop(&mut self) -> Option<T> {
            if self.len == 0 {
                return None;
            }
            self.len -= 1;
            unsafe { Some(self.data[self.len].assume_init_read()) }
        }

        pub fn get(&self, index: usize) -> Option<&T> {
            if index < self.len {
                unsafe { Some(&*self.data[index].as_ptr()) }
            } else {
                None
            }
        }

        pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
            if index < self.len {
                unsafe { Some(&mut *self.data[index].as_mut_ptr()) }
            } else {
                None
            }
        }

        pub fn as_slice(&self) -> &[T] {
            unsafe { core::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
        }

        pub fn as_mut_slice(&mut self) -> &mut [T] {
            unsafe { core::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.len) }
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

        pub fn clear(&mut self) {
            while self.pop().is_some() {}
        }
    }

    impl<T: Clone, const CAP: usize> Clone for Vec<T, CAP> {
        fn clone(&self) -> Self {
            let mut new_vec = Self::new();
            for item in self.as_slice() {
                let _ = new_vec.push(item.clone());
            }
            new_vec
        }
    }
}

pub mod hash_map {
    use core::hash::{Hash, Hasher};

    pub struct HashMap<K, V, const CAP: usize, H: Hasher> {
        entries: [Option<(K, V)>; CAP],
        len: usize,
        hasher: H,
    }

    impl<K, V, const CAP: usize, H: Hasher + Default> HashMap<K, V, CAP, H>
    where
        K: Eq + Hash + Clone,
        V: Clone,
    {
        pub fn new() -> Self {
            Self {
                entries: [const { None }; CAP],
                len: 0,
                hasher: H::default(),
            }
        }

        pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, &'static str> {
            if self.len >= CAP {
                return Err("HashMap full");
            }

            let index = self.hash_index(&key);
            let start_index = index;
            
            loop {
                match &mut self.entries[index] {
                    Some((k, v)) if *k == key => {
                        // Replace existing
                        let old_value = core::mem::replace(v, value);
                        return Ok(Some(old_value));
                    }
                    None => {
                        // Insert new
                        self.entries[index] = Some((key, value));
                        self.len += 1;
                        return Ok(None);
                    }
                    _ => {}
                }
                
                // Linear probing
                let next_index = (index + 1) % CAP;
                if next_index == start_index {
                    return Err("No space available");
                }
            }
        }

        pub fn get(&self, key: &K) -> Option<&V> {
            let index = self.hash_index(key);
            let start_index = index;
            
            loop {
                match &self.entries[index] {
                    Some((k, v)) if k == key => return Some(v),
                    None => return None,
                    _ => {}
                }
                
                let next_index = (index + 1) % CAP;
                if next_index == start_index {
                    return None;
                }
            }
        }

        fn hash_index(&self, key: &K) -> usize {
            let mut hasher = self.hasher;
            key.hash(&mut hasher);
            (hasher.finish() as usize) % CAP
        }

        pub fn len(&self) -> usize {
            self.len
        }

        pub fn is_empty(&self) -> bool {
            self.len == 0
        }
    }
}

pub mod binary_heap {
    use core::cmp::Ord;

    pub struct BinaryHeap<T: Ord, const CAP: usize> {
        data: [Option<T>; CAP],
        len: usize,
    }

    impl<T: Ord, const CAP: usize> BinaryHeap<T, CAP> {
        pub const fn new() -> Self {
            Self {
                data: [const { None }; CAP],
                len: 0,
            }
        }

        pub fn push(&mut self, item: T) -> Result<(), &'static str> {
            if self.len >= CAP {
                return Err("Heap full");
            }

            self.data[self.len] = Some(item);
            self.sift_up(self.len);
            self.len += 1;
            Ok(())
        }

        pub fn pop(&mut self) -> Option<T> {
            if self.len == 0 {
                return None;
            }

            let result = self.data[0].take();
            self.len -= 1;
            
            if self.len > 0 {
                self.data[0] = self.data[self.len].take();
                self.sift_down(0);
            }

            result
        }

        pub fn peek(&self) -> Option<&T> {
            self.data[0].as_ref()
        }

        fn sift_up(&mut self, mut index: usize) {
            while index > 0 {
                let parent = (index - 1) / 2;
                if self.data[index] <= self.data[parent] {
                    break;
                }
                self.data.swap(index, parent);
                index = parent;
            }
        }

        fn sift_down(&mut self, mut index: usize) {
            loop {
                let left = 2 * index + 1;
                let right = 2 * index + 2;
                let mut largest = index;

                if left < self.len && self.data[left] > self.data[largest] {
                    largest = left;
                }

                if right < self.len && self.data[right] > self.data[largest] {
                    largest = right;
                }

                if largest == index {
                    break;
                }

                self.data.swap(index, largest);
                index = largest;
            }
        }

        pub fn len(&self) -> usize {
            self.len
        }

        pub fn is_empty(&self) -> bool {
            self.len == 0
        }
    }
}

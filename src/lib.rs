// use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Default)]
enum Bucket<K: Eq, V> {
    #[default]
    Unoccupied,
    Deleted,
    Occupied(Pair<K, V>),
}

#[derive(Eq, Clone)]
struct Pair<K, V> {
    key: K,
    value: V,
}

impl<K: Eq, V> PartialEq for Pair<K, V> {
    fn eq(&self, other: &Self) -> bool {
        return self.key == other.key;
    }
}

pub struct HashMap<K, V>
where
    K: Hash + PartialEq + Eq,
{
    buckets: Vec<Bucket<K, V>>,
    len: usize,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq + PartialEq + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.ensure_capacity();

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let mut bucket = hash as usize % self.buckets.len();

        // linear search for unoccupied bucket
        while let Bucket::Occupied(p) = &self.buckets[bucket]
            && p.key != key
        {
            bucket = (bucket + 1) % self.buckets.len();
        }

        self.len += 1;
        let existing = std::mem::replace(
            &mut self.buckets[bucket],
            Bucket::Occupied(Pair { key, value }),
        );

        match existing {
            Bucket::Occupied(p) => Some(p.value),
            _ => None,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let expected_bucket = hash as usize % self.buckets.len();

        // linear search for bucket
        let mut bucket = expected_bucket;
        loop {
            match &self.buckets[bucket] {
                Bucket::Unoccupied => return None,
                Bucket::Deleted => bucket = (bucket + 1) % self.buckets.len(),
                Bucket::Occupied(p) => {
                    if key == &p.key {
                        return Some(&p.value);
                    }
                    bucket = (bucket + 1) % self.buckets.len()
                }
            }

            // we went through the full map
            if bucket == expected_bucket {
                break;
            }
        }
        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let expected_bucket = hash as usize % self.buckets.len();

        // linear search for bucket
        let mut bucket = expected_bucket;
        loop {
            match &self.buckets[bucket] {
                Bucket::Unoccupied => return None,
                Bucket::Deleted => bucket = (bucket + 1) % self.buckets.len(),
                Bucket::Occupied(p) => {
                    if key == &p.key {
                        let existing =
                            std::mem::replace(&mut self.buckets[bucket], Bucket::Deleted);
                        if let Bucket::Occupied(p) = existing {
                            return Some(p.value);
                        }
                    }
                    bucket = (bucket + 1) % self.buckets.len()
                }
            }

            // we went through the full map
            if bucket == expected_bucket {
                break;
            }
        }
        None
    }

    pub fn ensure_capacity(&mut self) {
        if self.len == self.buckets.len() {
            let mut new_buckets = Vec::with_capacity(self.buckets.len() * 2);
            new_buckets.resize_with(self.buckets.len() * 2, || Bucket::Unoccupied);
            let old_buckets = std::mem::replace(&mut self.buckets, new_buckets);

            // insert the old elements
            self.len = 0;
            for b in old_buckets {
                if let Bucket::Occupied(p) = b {
                    self.insert(p.key, p.value);
                }
            }
        }
    }

    pub fn capacity(&self) -> usize {
        self.buckets.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<K: Hash + Eq, V> Default for HashMap<K, V> {
    fn default() -> Self {
        let mut buckets = Vec::with_capacity(1024);
        buckets.resize_with(1024, || Bucket::Unoccupied);
        Self { buckets, len: 0 }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_many() {
        // given
        let mut map = HashMap::new();

        // when
        for i in 0..1_0000 {
            map.insert(i.to_string(), i);
        }
    }

    #[test]
    fn test_get() {
        #[derive(Debug, Eq)]
        struct KeyWithFixedHash<K: PartialEq + Eq> {
            hash: [u8; 4],
            key: K,
        }

        impl<V: Eq> Hash for KeyWithFixedHash<V> {
            fn hash<H: Hasher>(&self, state: &mut H) {
                state.write(&self.hash)
            }
        }

        impl<K: PartialEq + Eq> PartialEq for KeyWithFixedHash<K> {
            fn eq(&self, other: &Self) -> bool {
                self.key == other.key
            }
        }

        // given
        let mut map = HashMap::new();
        for i in 0..10 {
            map.insert(
                KeyWithFixedHash {
                    hash: [1, 2, 3, 4],
                    key: i,
                },
                i,
            );
        }

        // when/then
        assert_eq!(
            Some(&9),
            map.get(&KeyWithFixedHash {
                hash: [1, 2, 3, 4],
                key: 9
            }),
            "correctly resolves collisions"
        );
        assert_eq!(
            None,
            map.get(&KeyWithFixedHash {
                hash: [1, 2, 3, 4],
                key: 10,
            }),
            "doesnt loop infinitely if not exists and same hash"
        );
        assert_eq!(
            None,
            map.get(&KeyWithFixedHash {
                hash: [0, 0, 0, 0],
                key: 10,
            }),
            "finds nothing if hash doesn't match exists"
        );
    }

    #[test]
    fn test_integration() {
        // given
        let mut map = HashMap::new();
        for i in 0..10 {
            map.insert(i.to_string(), i);
        }

        // when/then
        assert_eq!(
            Some(&9),
            map.get(&"9".to_string()),
            "finds existing element"
        );
        assert_eq!(
            Some(9),
            map.remove(&"9".to_string()),
            "removes existing element"
        );
        assert_eq!(
            None,
            map.get(&"9".to_string()),
            "does not find removed element"
        );
        assert_eq!(
            Some(1),
            map.insert("1".to_string(), 12),
            "returns existing element on insertion"
        );
        assert_eq!(
            Some(&12),
            map.get(&"1".to_string()),
            "finds inserted element"
        );
    }
}

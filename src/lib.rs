#![feature(test)]
extern crate test;

// use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Default)]
enum Bucket<K, V> {
    #[default]
    Unoccupied,
    Deleted,
    Occupied(Entry<K, V>),
}

struct Entry<K, V> {
    key: K,
    value: V,
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
    K: Hash + Eq + PartialEq,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.ensure_capacity();

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let mut pos = hash as usize % self.buckets.len();

        // quadratic probing for unoccupied bucket
        let mut probe_count = 0usize;
        while let Bucket::Occupied(p) = &self.buckets[pos]
            && p.key != key
        {
            probe_count += 1;
            pos = (pos + probe_count.pow(2)) % self.buckets.len();
        }

        self.len += 1;
        let existing = std::mem::replace(
            &mut self.buckets[pos],
            Bucket::Occupied(Entry { key, value }),
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
        let expected_pos = hash as usize % self.buckets.len();

        // quadratic probing for bucket
        let mut pos = expected_pos;
        let mut probe_count = 0usize;
        loop {
            probe_count += 1;
            match &self.buckets[pos] {
                Bucket::Unoccupied => return None,
                Bucket::Deleted => pos = (pos + probe_count.pow(2)) % self.buckets.len(),
                Bucket::Occupied(p) => {
                    if key == &p.key {
                        return Some(&p.value);
                    }
                    pos = (pos + probe_count.pow(2)) % self.buckets.len()
                }
            }

            // we went through the full map
            if pos == expected_pos {
                break;
            }
        }
        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let expected_pos = hash as usize % self.buckets.len();

        // quadratic probing for bucket
        let mut pos = expected_pos;
        let mut probe_count = 0usize;
        loop {
            probe_count += 1;
            match &self.buckets[pos] {
                Bucket::Unoccupied => return None,
                Bucket::Deleted => pos = (pos + probe_count.pow(2)) % self.buckets.len(),
                Bucket::Occupied(p) => {
                    if key == &p.key {
                        let existing = std::mem::replace(&mut self.buckets[pos], Bucket::Deleted);
                        if let Bucket::Occupied(p) = existing {
                            return Some(p.value);
                        }
                    }
                    pos = (pos + probe_count.pow(2)) % self.buckets.len()
                }
            }

            // we went through the full map
            if pos == expected_pos {
                break;
            }
        }
        None
    }

    pub fn ensure_capacity(&mut self) {
        const LOAD_FACTOR_MAX: u64 = 65;
        let load_factor = self.len * 100 / self.buckets.len();
        if load_factor as u64 >= LOAD_FACTOR_MAX {
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

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        self.into_iter()
    }
}

impl<K: Hash + Eq, V> Default for HashMap<K, V> {
    fn default() -> Self {
        let mut buckets = Vec::with_capacity(1024);
        buckets.resize_with(1024, || Bucket::Unoccupied);
        Self { buckets, len: 0 }
    }
}

impl<'a, K: Hash + PartialEq + Eq, V> IntoIterator for &'a HashMap<K, V> {
    type Item = Pair<&'a K, &'a V>;

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            buckets: self.buckets.iter(),
        }
    }
}

pub struct Iter<'a, K, V> {
    buckets: std::slice::Iter<'a, Bucket<K, V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = Pair<&'a K, &'a V>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(bucket) = self.buckets.next() {
            if let Bucket::Occupied(item) = bucket {
                return Some(Pair {
                    key: &item.key,
                    value: &item.value,
                });
            }
        }
        None
    }
}

impl<K: Hash + PartialEq + Eq, V> IntoIterator for HashMap<K, V> {
    type Item = Pair<K, V>;

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            buckets: self.buckets.into_iter(),
        }
    }
}

pub struct IntoIter<K, V> {
    buckets: std::vec::IntoIter<Bucket<K, V>>,
}

#[derive(Debug, PartialEq)]
pub struct Pair<K, V> {
    pub key: K,
    pub value: V,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = Pair<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(bucket) = self.buckets.next() {
            if let Bucket::Occupied(item) = bucket {
                return Some(Pair {
                    key: item.key,
                    value: item.value,
                });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_insert_many() {
        // given
        let mut map = HashMap::new();

        // when
        for i in 0..1_0000 {
            map.insert(i.to_string(), i);
        }
    }

    #[bench]
    fn bench_insert_many(b: &mut Bencher) {
        let mut map = HashMap::new();
        b.iter(|| {
            for i in 0..1_00000 {
                map.insert(i.to_string(), i);
            }
        });
    }

    #[bench]
    fn bench_insert_many_std(b: &mut Bencher) {
        let mut map = std::collections::HashMap::new();
        b.iter(|| {
            for i in 0..1_00000 {
                map.insert(i.to_string(), i);
            }
        });
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

    #[test]
    fn test_into_iter() {
        // given
        let mut want_pairs: Vec<_> = (0..1_0000)
            .map(|i| Pair {
                key: i.to_string(),
                value: i,
            })
            .collect();

        let mut map = HashMap::new();
        for p in want_pairs.iter() {
            map.insert(p.key.clone(), p.value);
        }

        // when
        let mut got_pairs: Vec<_> = map.into_iter().collect();

        // then
        want_pairs.sort_by(|a, b| a.key.cmp(&b.key));
        got_pairs.sort_by(|a, b| a.key.cmp(&b.key));
        assert_eq!(want_pairs, got_pairs);
    }

    #[test]
    fn test_iter() {
        // given
        let mut map = HashMap::new();
        for i in 0..1_0000 {
            map.insert(i.to_string(), i);
        }

        // when
        let got_pairs: Vec<_> = map.iter().collect();

        // then
        for i in 0..1_0000 {
            assert!(got_pairs.contains(&Pair {
                key: &i.to_string(),
                value: &i
            }));
        }
    }
}

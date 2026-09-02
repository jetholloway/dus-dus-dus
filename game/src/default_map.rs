use std::collections::HashMap;
use std::hash::Hash;

pub struct DefaultMap<K, V> {
    map: HashMap<K, V>,
}

impl<K, V> DefaultMap<K, V> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.map.values()
    }
}

impl<K: Clone + Eq + PartialEq + Hash, V: Default> DefaultMap<K, V> {
    pub fn at(&mut self, index: K) -> &mut V {
        if !self.map.contains_key(&index) {
            let _ = self.map.insert(index.clone(), V::default());
        }

        self.map.get_mut(&index).unwrap()
    }
}

impl<K, V> Default for DefaultMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

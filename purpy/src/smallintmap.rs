#[derive(Debug)]
pub struct SmallIntMap<K, V>
where
    K: Into<usize>,
{
    // keys is currently unused, but useful for typing, and may be useful later
    _keys: Vec<Option<K>>,
    values: Vec<Option<V>>,
}

impl<K, V> SmallIntMap<K, V>
where
    K: Into<usize>,
{
    pub fn new() -> Self {
        SmallIntMap {
            _keys: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn insert(&mut self, k: K, v: V) {
        let k: usize = k.into();
        while k >= self.values.len() {
            self.values.push(None);
        }
        self.values[k] = Some(v);
    }

    pub fn get(&self, k: K) -> Option<&V> {
        let k: usize = k.into();
        self.values.get(k).and_then(|ov| ov.as_ref())
    }

    pub fn get_mut(&mut self, k: K) -> Option<&mut V> {
        let k: usize = k.into();
        self.values.get_mut(k).and_then(|ov| ov.as_mut())
    }
}

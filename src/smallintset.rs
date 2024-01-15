pub struct SmallIntSet<T> {
    items: Vec<T>,
}

impl<T> SmallIntSet<T>
where
    T: PartialEq,
{
    pub fn new() -> Self {
        SmallIntSet { items: Vec::new() }
    }

    pub fn insert(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn contains(&self, item: T) -> bool {
        self.items.contains(&item)
    }

    pub fn remove(&mut self, item: T) {
        let mut len = self.items.len();
        let mut i = 0;
        while i < len {
            if self.items[i] == item {
                len -= 1;
                self.items.swap(i, len);
            }
        }
        self.items.truncate(len);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }
}

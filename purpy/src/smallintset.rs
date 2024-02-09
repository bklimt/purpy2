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

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }
}

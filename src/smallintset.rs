pub struct SmallIntSet {
    items: Vec<bool>,
}

impl SmallIntSet {
    pub fn new() -> Self {
        SmallIntSet { items: Vec::new() }
    }

    pub fn insert<T>(&mut self, item: T)
    where
        T: Into<usize>,
    {
        let item = item.into();
        if self.items.len() <= item {
            self.items.resize(item + 1, false);
        }
        self.items[item] = true;
    }

    pub fn contains<T>(&self, item: T) -> bool
    where
        T: Into<usize>,
    {
        let item = item.into();
        *self.items.get(item).unwrap_or(&false)
    }

    pub fn remove<T>(&mut self, item: T)
    where
        T: Into<usize>,
    {
        let item = item.into();
        if let Some(b) = self.items.get_mut(item) {
            *b = false;
        }
    }
}

use std::collections::HashSet;

pub struct VecSet<T> {
    set: HashSet<T>,
    vec: Vec<T>,
}

impl<T> VecSet<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    pub fn new() -> Self {
        Self {
            set: HashSet::new(),
            vec: Vec::new(),
        }
    }
    pub fn insert(&mut self, elem: T) {
        assert_eq!(self.set.len(), self.vec.len());
        let was_new = self.set.insert(elem.clone());
        if was_new {
            self.vec.push(elem);
        }
    }
    pub fn remove_front(&mut self) -> Option<T> {
        assert_eq!(self.set.len(), self.vec.len());
        if self.set.len() < 1 {
            return None;
        }
        let index = 0;
        let elem = self.vec.swap_remove(index);
        self.set.remove(&elem);
        Some(elem)
    }
    pub fn remove(&mut self, elem: &T) {
        if let Some(index) = self.vec.iter().position(|f| f == elem) {
            self.vec.swap_remove(index);
        }
        self.set.remove(elem);
    }
    pub fn is_empty(&self) -> bool {
        assert_eq!(self.set.len(), self.vec.len());
        self.vec.is_empty()
    }
}

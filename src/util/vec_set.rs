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
        let elem = self.vec.remove(index);
        self.set.remove(&elem);
        Some(elem)
    }
    pub fn remove(&mut self, elem: &T) {
        if let Some(index) = self.vec.iter().position(|f| f == elem) {
            self.vec.swap_remove(index);
        }
        self.set.remove(elem);
    }
    pub fn contains(&self, elem: &T) -> bool {
        self.set.contains(elem)
    }
    pub fn is_empty(&self) -> bool {
        assert_eq!(self.set.len(), self.vec.len());
        self.vec.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_test() {
        let set = VecSet::<u32>::new();
        assert!(set.is_empty());
    }

    #[test]
    fn insert_contains_test() {
        let mut set = VecSet::<u32>::new();
        set.insert(45);
        assert!(set.contains(&45));
    }

    #[test]
    fn remove_test() {
        let mut set = VecSet::<u32>::new();
        set.insert(45);
        set.remove(&45);
        assert!(set.is_empty());
    }

    #[test]
    fn remove_front_test() {
        let mut set = VecSet::<u32>::new();
        set.insert(45);
        set.insert(67);
        set.insert(89);
        set.insert(10);
        assert!(set.remove_front().is_some());
        assert!(set.remove_front().unwrap() == 67);
    }
}

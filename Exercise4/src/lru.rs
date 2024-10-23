use std::{collections::{BTreeSet, HashMap}, hash::Hash};

#[derive(Debug, Clone)]
pub struct LRUUse<T> {
    time: usize,
    value: T,
}

impl<T> PartialEq for LRUUse<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl<T> Eq for LRUUse<T> { }

impl<T> PartialOrd for LRUUse<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.time.cmp(&other.time))
    }
}

impl<T> Ord for LRUUse<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

impl<T> LRUUse<T> {
    #[inline]
    fn new(time: usize, value: T) -> Self {
        Self { time, value }
    }

    #[inline]
    pub fn get_val(&self) -> &T {
        &self.value
    }

    #[inline]
    pub fn get_time(&self) -> usize {
        self.time
    }
}

#[derive(Debug)]
pub struct Lru<T: PartialEq + Eq + Hash + Clone> {
    lru: BTreeSet<LRUUse<T>>,
    use_time: HashMap<T, usize>,
}

impl<T: PartialEq + Eq + Hash + Clone> Lru<T> {
    #[inline]
    pub fn new() -> Self {
        Self { lru: BTreeSet::new(), use_time: HashMap::new() }
    }

    pub fn use_val(&mut self, value: T, time: usize) {
        if let Some(last_use) = self.use_time.get_mut(&value) {
            self.lru.remove(&LRUUse::new(*last_use, value.clone()));
            *last_use = time;
        }
        else {
            self.use_time.insert(value.clone(), time);
        }
        self.lru.insert(LRUUse::new(time, value));
    }

    #[inline]
    pub fn pop_lru(&mut self) -> Option<T> {
        self.lru.pop_first().map(|v| v.value)
    }

    #[inline]
    pub fn lru_iter(&self) -> &BTreeSet<LRUUse<T>> {
        &self.lru
    }

    #[inline]
    pub fn remove_lru(&mut self, value: &LRUUse<T>) {
        self.lru.remove(value);
        // self.use_time.remove(&value.value);
    }

    #[inline]
    pub fn remove_val(&mut self, value: T) {
        if let Some(last_use) = self.use_time.remove(&value) {
            self.lru.remove(&LRUUse::new(last_use, value.clone()));
        }
    }
}

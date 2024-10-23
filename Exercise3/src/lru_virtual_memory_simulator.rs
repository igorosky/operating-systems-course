use std::collections::{BTreeSet, HashMap};

use crate::virtual_memory_simulator::VirtualMemorySimulator;

#[derive(Debug, Clone)]
pub struct LRUVirtualMemorySimulator {
    memory_size: usize,
    page_faults: usize,
    pages: HashMap<usize, bool>,
    oldest_access: BTreeSet<(usize, usize)>,
    latest_access: HashMap<usize, usize>,
    access_counter: usize,
}

impl VirtualMemorySimulator for LRUVirtualMemorySimulator {
    #[inline]
    fn new(memory_size: usize) -> Self {
        if memory_size == 0 {
            panic!("Cannot simulate without memory");
        }
        Self { memory_size, page_faults: 0, pages: HashMap::new(), oldest_access: BTreeSet::new(), latest_access: HashMap::new(), access_counter: 0 }
    }
    
    #[inline]
    fn finalize(self) -> usize {
        assert!(self.pages.into_iter().filter(|(_, present)| *present).count() <= self.memory_size, "Memory simulation was invalidated");
        self.page_faults
    }
    
    fn get_page(&mut self, page_id: usize) {
        if let Some(x) = self.latest_access.get(&page_id) {
            self.oldest_access.remove(&(*x, page_id));
        }
        match self.pages.get(&page_id).cloned() {
            Some(true) => (),
            Some(false) | None => {
                self.get_space();
                self.pages.insert(page_id, true);
                self.page_faults += 1;
            }
        }
        self.latest_access.insert(page_id, self.access_counter);
        self.oldest_access.insert((self.access_counter, page_id));
        self.access_counter += 1;
    }
}

impl LRUVirtualMemorySimulator {
    #[inline]
    fn get_space(&mut self) {
        if self.oldest_access.len() == self.memory_size {
            *self.pages.get_mut(&self.oldest_access.pop_first().unwrap().1).unwrap() = false;
        }
    }
}

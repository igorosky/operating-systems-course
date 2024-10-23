use std::collections::{HashMap, LinkedList};

use crate::virtual_memory_simulator::VirtualMemorySimulator;

#[derive(Debug, Clone)]
pub struct ApproximatedLRUVirtualMemorySimulator {
    memory_size: usize,
    page_faults: usize,
    pages: HashMap<usize, bool>,
    pages_fifo: LinkedList<usize>,
    page_access_bit: HashMap<usize, bool>,
}

impl VirtualMemorySimulator for ApproximatedLRUVirtualMemorySimulator {
    #[inline]
    fn new(memory_size: usize) -> Self {
        if memory_size == 0 {
            panic!("Cannot simulate without memory");
        }
        Self { memory_size, page_faults: 0, pages: HashMap::new(), pages_fifo: LinkedList::new(), page_access_bit: HashMap::new() }
    }
    
    #[inline]
    fn finalize(self) -> usize {
        assert!(self.pages.into_iter().filter(|(_, present)| *present).count() <= self.memory_size, "Memory simulation was invalidated");
        self.page_faults
    }
    
    fn get_page(&mut self, page_id: usize) {
        match self.pages.get(&page_id).cloned() {
            Some(true) => (),
            Some(false) | None => {
                self.get_space();
                self.pages.insert(page_id, true);
                self.pages_fifo.push_back(page_id);
                self.page_faults += 1;
            }
        }
        self.page_access_bit.insert(page_id, true);
    }
}

impl ApproximatedLRUVirtualMemorySimulator {
    #[inline]
    fn get_space(&mut self) {
        if self.pages_fifo.len() == self.memory_size {
            while self.page_access_bit[self.pages_fifo.front().unwrap()] {
                let page_id = self.pages_fifo.pop_front().unwrap();
                self.page_access_bit.insert(page_id, false);
                self.pages_fifo.push_back(page_id);
            }
            *self.pages.get_mut(&self.pages_fifo.pop_front().unwrap()).unwrap() = false;
        }
    }
}

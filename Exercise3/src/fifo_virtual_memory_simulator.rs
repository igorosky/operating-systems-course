use std::collections::{HashMap, LinkedList};

use crate::virtual_memory_simulator::VirtualMemorySimulator;

#[derive(Debug, Clone)]
pub struct FIFOVirtualMemorySimulator {
    memory_size: usize,
    page_faults: usize,
    pages: HashMap<usize, bool>,
    pages_fifo: LinkedList<usize>,
}

impl VirtualMemorySimulator for FIFOVirtualMemorySimulator {
    #[inline]
    fn new(memory_size: usize) -> Self {
        if memory_size == 0 {
            panic!("Cannot simulate without memory");
        }
        Self { memory_size, page_faults: 0, pages: HashMap::new(), pages_fifo: LinkedList::new() }
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
    }
}

impl FIFOVirtualMemorySimulator {
    #[inline]
    fn get_space(&mut self) {
        if self.pages_fifo.len() == self.memory_size {
            *self.pages.get_mut(&self.pages_fifo.pop_front().unwrap()).unwrap() = false;
        }
    }
}

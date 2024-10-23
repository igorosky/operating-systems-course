use std::collections::HashMap;

use crate::virtual_memory_simulator::VirtualMemorySimulator;

use rand::Rng;

#[derive(Debug, Clone)]
pub struct RandVirtualMemorySimulator {
    memory_size: usize,
    page_faults: usize,
    pages: HashMap<usize, bool>,
    loaded_pages: Vec<usize>,
}

impl VirtualMemorySimulator for RandVirtualMemorySimulator {
    #[inline]
    fn new(memory_size: usize) -> Self {
        if memory_size == 0 {
            panic!("Cannot simulate without memory");
        }
        Self { memory_size, page_faults: 0, pages: HashMap::new(), loaded_pages: Vec::new() }
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
                self.loaded_pages.push(page_id);
                self.page_faults += 1;
            }
        }
    }
}

impl RandVirtualMemorySimulator {
    #[inline]
    fn get_space(&mut self) {
        let loaded_pages_len = self.loaded_pages.len();
        if loaded_pages_len == self.memory_size {
            self.loaded_pages.swap(rand::thread_rng().gen_range(0..loaded_pages_len), loaded_pages_len - 1);
            *self.pages.get_mut(&self.loaded_pages.pop().unwrap()).unwrap() = false;
        }
    }
}

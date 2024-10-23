use std::collections::{BTreeSet, HashMap, LinkedList};

use crate::virtual_memory_simulator::VirtualMemorySimulator;

#[derive(Debug, Clone)]
pub struct OPTVirtualMemorySimulator {
    memory_size: usize,
    page_faults: usize,
    pages: HashMap<usize, bool>,
    next_access: LinkedList<usize>,
    latest_access: BTreeSet<(usize, usize)>,
}

impl VirtualMemorySimulator for OPTVirtualMemorySimulator {
    #[inline]
    fn new(memory_size: usize) -> Self {
        if memory_size == 0 {
            panic!("Cannot simulate without memory");
        }
        Self { memory_size, page_faults: 0, pages: HashMap::new(), next_access: LinkedList::new(), latest_access: BTreeSet::new() }
    }
    
    #[inline]
    fn finalize(self) -> usize {
        assert!(self.pages.into_iter().filter(|(_, present)| *present).count() <= self.memory_size, "Memory simulation was invalidated");
        self.page_faults
    }
    
    fn get_page(&mut self, page_id: usize) {
        match self.pages.get(&page_id).cloned() {
            Some(true) => {
                self.latest_access.pop_first();
            }
            Some(false) | None => {
                self.get_space();
                self.pages.insert(page_id, true);
                self.page_faults += 1;
            }
        }
        self.latest_access.insert((self.next_access.pop_front().unwrap(), page_id));
    }

    fn create_and_simulate<T: IntoIterator<Item = usize>>(memory_size: usize, memory_accesses: T) -> usize {
        let mut memory_accesses_copy = LinkedList::new();
        let mut next_access = Vec::new();
        let mut previous_access = HashMap::new();
        for (i, v) in memory_accesses.into_iter().enumerate() {
            memory_accesses_copy.push_back(v);
            next_access.push(usize::MAX);
            if let Some(pos) = previous_access.get_mut(&v) {
                next_access[*pos] = i;
                *pos = i;
            }
            else {
                previous_access.insert(v, i);
            }
        }
        drop(previous_access);
        let mut vm = Self::new(memory_size);
        vm.next_access = LinkedList::from_iter(next_access.into_iter());
        memory_accesses_copy.into_iter().for_each(|page_id| vm.get_page(page_id));
        vm.finalize()
    }
}

impl OPTVirtualMemorySimulator {
    #[inline]
    fn get_space(&mut self) {
        if self.latest_access.len() == self.memory_size {
            *self.pages.get_mut(&self.latest_access.pop_last().unwrap().1).unwrap() = false;
        }
    }
}

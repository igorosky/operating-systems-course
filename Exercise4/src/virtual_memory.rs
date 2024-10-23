use std::collections::HashSet;

use crate::{lru::Lru, process::Process, reminder_sorter::ProcessSortableReminders};

pub mod equal_allocation;
pub mod proportional;
pub mod page_fault_frequency;
pub mod working_set_size;

pub trait VirtualMemory: Sync {
    fn emulate(&self, processes: Vec<Process>) -> Vec<usize>;
}

enum EmulationStepResult {
    Done(usize),
    NoMemoryAvailable,
    NoMoreWork,
}

struct ProcessWrapper {
    process: Process,
    lru: Lru<usize>,
    used_space: usize,
    available_size: usize,
    abstract_time_counter: usize,
}

impl From<Process> for ProcessWrapper {
    fn from(process: Process) -> Self {
        Self { process, lru: Lru::new(), used_space: 0, available_size: 0, abstract_time_counter: 0 }
    }
}

impl ProcessWrapper {
    #[inline]
    fn set_available_memory_builder(mut self, space: usize) -> Self {
        self.available_size = space;
        self
    }

    #[inline]
    fn get_available_memory(&self) -> usize {
        self.available_size
    }

    fn next(&mut self, memory: &mut HashSet<usize>) -> EmulationStepResult {
        if !self.process.is_empty() && self.available_size == 0 {
            return EmulationStepResult::NoMemoryAvailable;
        }
        match self.process.next() {
            Some(accesses) => {
                let mut ans = 0;
                for access in accesses {
                    if !memory.contains(&access) {
                        while self.available_size <= self.used_space {
                            #[cfg(not(debug_assertions))]
                            memory.remove(&self.lru.pop_lru().unwrap());
                            #[cfg(debug_assertions)]
                            if !memory.remove(&self.lru.pop_lru().unwrap()) {
                                panic!("No memory released");
                            }
                            self.used_space -= 1;
                        }
                        self.used_space += 1;
                        memory.insert(access);
                        ans += 1;
                    }
                    self.lru.use_val(access, self.abstract_time_counter);
                    self.abstract_time_counter += 1;
                }
                if self.process.is_empty() {
                    #[cfg(not(debug_assertions))]
                    self.process.get_used_pages()
                        .into_iter()
                        .for_each(|v| {
                            memory.remove(v);
                        });
                    #[cfg(debug_assertions)]
                    if self.used_space != self.process.get_used_pages()
                        .iter()
                        .map(|v| memory.remove(v))
                        .filter(|v| *v)
                        .count() {
                            panic!("Not all memory was released")
                        }
                }
                EmulationStepResult::Done(ans)
            }
            None => EmulationStepResult::NoMoreWork,
        }
    }
}

impl ProcessSortableReminders for ProcessWrapper {
    #[inline]
    fn get_page_count(&self) -> usize {
        self.process.get_page_count()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.process.is_empty()
    }

    #[inline]
    fn set_available_memory(&mut self, space: usize) {
        self.available_size = space;
    }
}

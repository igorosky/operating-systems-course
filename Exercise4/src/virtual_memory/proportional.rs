use std::{collections::HashSet, usize};

use crate::{process::Process, reminder_sorter::{ProcessSortableReminders, ReminderSorter}};

use super::{EmulationStepResult, ProcessWrapper, VirtualMemory};

#[derive(Debug, Clone, Hash)]
pub(crate) struct Proportional {
    memory_size: usize,
}

impl Proportional {
    #[inline]
    pub(crate) fn new(memory_size: usize) -> Self {
        Self { memory_size }
    }
}

impl VirtualMemory for Proportional {
    fn emulate(&self, processes: Vec<Process>) -> Vec<usize> {
        let mut processes: Vec<ProcessWrapper> = processes
            .into_iter()
            .map(ProcessWrapper::from)
            .collect();
        let mut memory = HashSet::new();
        let mut ans = vec![0; processes.len()];
        let mut total_amount_of_pages: usize = processes.iter()
            .map(|process| process.get_page_count())
            .sum();
        let mut reminder_sorter = ReminderSorter::new(self.memory_size, processes.len());
        reminder_sorter.set_frame_counts(total_amount_of_pages, &mut processes);
        let mut completed_processes = 0;
        while completed_processes < processes.len() {
            for i in 0..processes.len() {
                match processes[i].next(&mut memory) {
                    EmulationStepResult::Done(v) => {
                        #[cfg(debug_assertions)]
                        if memory.len() > self.memory_size {
                            panic!("Memory overflow (proportional) - used memory: {}, available memory: {}", memory.len(), self.memory_size);
                        }
                        ans[i] += v;
                        if processes[i].is_empty() {
                            completed_processes += 1;
                            total_amount_of_pages -= processes[i].get_page_count();
                            reminder_sorter.set_frame_counts(total_amount_of_pages, &mut processes);
                            for process in processes.iter_mut().filter(|v| !v.is_empty()) {
                                while process.used_space > process.available_size {
                                    memory.remove(&process.lru.pop_lru().unwrap());
                                    process.used_space -= 1;
                                }
                            }
                        }
                    },
                    EmulationStepResult::NoMemoryAvailable => (),
                    EmulationStepResult::NoMoreWork => (),
                }
            }
        }
        ans
    }
}

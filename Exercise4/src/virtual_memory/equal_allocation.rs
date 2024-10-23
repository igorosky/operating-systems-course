use std::collections::HashSet;

use crate::{process::Process, reminder_sorter::ProcessSortableReminders};

use super::{EmulationStepResult, ProcessWrapper, VirtualMemory};

#[derive(Debug, Clone, Hash)]
pub(crate) struct EqualAllocation {
    memory_size: usize,
}

impl EqualAllocation {
    #[inline]
    pub(crate) fn new(memory_size: usize) -> Self {
        Self { memory_size }
    }

    #[inline]
    fn div_into_nth(count: usize, elements: usize, n: usize) -> usize {
        count / elements + (n < count % elements) as usize
    }
}

impl VirtualMemory for EqualAllocation {
    fn emulate(&self, processes: Vec<Process>) -> Vec<usize> {
        let mut memory = HashSet::new();
        let mut ans = vec![0; processes.len()];
        let mut next_to_process = processes.len().min(self.memory_size);
        let mut current_processes = Vec::from_iter(
            (0..next_to_process)
            .map(|i| 
                (i, ProcessWrapper::from(processes[i].clone())
                    .set_available_memory_builder(
                        Self::div_into_nth(
                            self.memory_size,
                            next_to_process,
                            i
                        )
                    ))
            )
        );
        while !current_processes.is_empty() {
            let mut i = 0;
            while i < current_processes.len() {
                match current_processes[i].1.next(&mut memory) {
                    EmulationStepResult::Done(page_faults) => {
                        #[cfg(debug_assertions)]
                        if memory.len() > self.memory_size {
                            panic!("Memory overflow (equal) - used memory: {}, available memory: {}", memory.len(), self.memory_size);
                        }
                        ans[current_processes[i].0] += page_faults;
                        if current_processes[i].1.is_empty() {
                            if next_to_process < processes.len() {
                                let mut wrapper = ProcessWrapper::from(
                                    processes[next_to_process].clone())
                                    .set_available_memory_builder(current_processes[i].1.get_available_memory());
                                std::mem::swap(&mut wrapper, &mut current_processes[i].1);
                                next_to_process += 1;
                            }
                            else {
                                current_processes.swap_remove(i);
                                let current_processes_len = current_processes.len();
                                for (j, (_, process)) in current_processes.iter_mut().enumerate() {
                                    process
                                        .set_available_memory(
                                            Self::div_into_nth(
                                                self.memory_size,
                                                current_processes_len,
                                                j
                                        ));
                                }
                            }
                            continue;
                        }
                    }
                    EmulationStepResult::NoMemoryAvailable | EmulationStepResult::NoMoreWork => (),
                }
                i += 1;
            }
        }
        ans
    }
}

use std::collections::LinkedList;

use getset::{CopyGetters, Getters, MutGetters};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::process::Process;

#[derive(Debug, Clone, CopyGetters)]
pub struct ProcessorStatistics {
    #[getset(get_copy = "pub")]
    processes_count: usize,
    #[getset(get_copy = "pub")]
    used_resources: usize,
}

#[derive(Debug, Clone, Getters, CopyGetters, MutGetters)]
pub struct ProcessorWrapper {
    #[getset(get = "pub", get_mut = "pub")]
    processor: Processor,
    #[getset(get_copy = "pub")]
    load_requests: usize,
    #[getset(get = "pub", get_mut = "pub")]
    processes_queue: LinkedList<Process>,
    finished_processes: Vec<Process>,
}

impl ProcessorWrapper {
    #[inline]
    pub fn new(processor: Processor) -> Self {
        Self {
            processor,
            load_requests: 0,
            processes_queue: LinkedList::new(),
            finished_processes: Vec::new(),
        }
    }

    #[inline]
    pub fn work(&mut self) -> bool {
        self.finished_processes.append(&mut self.processor.work());
        self.processor.get_processes_count() != 0// || !self.processes_queue.is_empty()
    }

    #[inline]
    pub fn get_processor_load(&mut self) -> (f64, usize) {
        self.load_requests += 1;
        self.self_get_processor_load()
    }

    #[inline]
    pub fn self_get_processor_load(&mut self) -> (f64, usize) {
        (
            self.processor.used_resources as f64 / self.processor.resources as f64,
            self.processor.get_available_resources()
        )
    }

    pub fn give_process_away(&mut self, max_requirements: usize, minimal_left: f64) -> Option<Process> {
        if !self.processes_queue.is_empty() {
            if self.processes_queue.front().unwrap().required_resources() <= max_requirements {
                return self.processes_queue.pop_front();
            }
            if self.processes_queue.back().unwrap().required_resources() <= max_requirements {
                return self.processes_queue.pop_back();
            }
        }
        let minimal_load_left = (self.processor.resources as f64 * minimal_left) as usize;
        let maximum_process_taken_size = self.processor.used_resources.checked_sub(minimal_load_left);
        if let Some(mut maximum_process_taken_size) = maximum_process_taken_size {
            maximum_process_taken_size = maximum_process_taken_size.min(max_requirements);
            let process_to_give_away = self.processor.processes.par_iter()
                .enumerate()
                .find_first(|(_, process)|
                    process.required_resources() <= maximum_process_taken_size
                )
                .map(|(i, _)| i);
            if let Some(process_to_give_away) = process_to_give_away {
                let mut process = self.processor.processes.swap_remove(process_to_give_away);
                process.send();
                return Some(process);
            }
        }
        None
    }

    #[inline]
    pub fn get_statistics(&self) -> ProcessorStatistics {
        ProcessorStatistics {
            processes_count: self.processor.processes.len(),
            used_resources: self.processor.used_resources,
        }
    }

    // #[inline]
    // pub fn get_finished_processes(&self) -> &[Process] {
    //     &self.finished_processes
    // }

    #[inline]
    pub fn finish_and_get_finished_processes(self) -> Vec<Process> {
        self.finished_processes
    }
}

#[derive(Debug, Clone, CopyGetters)]
pub struct Processor {
    #[getset(get_copy = "pub")]
    resources: usize,
    #[getset(get_copy = "pub")]
    used_resources: usize,
    processes: Vec<Process>,
}

impl Processor {
    #[inline]
    pub fn new(resources: usize) -> Self {
        Self {
            resources,
            used_resources: 0,
            processes: Vec::new(),
        }
    }

    #[inline]
    fn work(&mut self) -> Vec<Process> {
        let efficiency = (self.resources as f64 / self.used_resources as f64).min(1.0);
        self.processes.par_iter_mut()
            .map(|process| {
                process.work(efficiency);
                process
            })
            .enumerate()
            .filter(|(_, process)| process.is_done())
            .map(|(i, _)| i)
            .collect::<Vec<usize>>()
            .into_iter()
            .rev()
            .map(|i| self.processes.swap_remove(i))
            .collect()
    }

    #[inline]
    pub(super) fn spawn_process(&mut self, process: Process) {
        self.used_resources += process.required_resources();
        self.processes.push(process);
    }

    #[inline]
    pub fn get_available_resources(&self) -> usize {
        self.resources.saturating_sub(self.used_resources)
    }

    #[inline]
    fn get_processes_count(&self) -> usize {
        self.processes.len()
    }
}

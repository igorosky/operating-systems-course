use std::collections::{HashSet, LinkedList};

use crate::{lru::Lru, process::Process, reminder_sorter::{ProcessSortableReminders, ReminderSorter}};

use super::VirtualMemory;

#[derive(Debug, Clone, Hash)]
pub(crate) struct PageFaultFrequency {
    memory_size: usize,
    minimal_amount_of_page_faults: usize,
    maximum_amount_of_page_faults: usize,
    period: usize,
}

impl PageFaultFrequency {
    #[inline]
    pub(crate) fn new(
        memory_size: usize,
        minimal_amount_of_page_faults: usize,
        maximum_amount_of_page_faults: usize,
        period: usize,
    ) -> Self {
        Self {
            memory_size,
            minimal_amount_of_page_faults,
            maximum_amount_of_page_faults,
            period,
        }
    } 
}

struct ProcessWrapperPFF {
    process: Process,
    lru: Lru<usize>,
    page_faults_list: LinkedList<bool>,
    page_faults_count: usize,
    desired_space: usize,
    used_memory: usize,
}

impl From<Process> for ProcessWrapperPFF {
    fn from(process: Process) -> Self {
        ProcessWrapperPFF {
            process,
            lru: Lru::new(),
            page_faults_list: LinkedList::new(),
            page_faults_count: 0,
            desired_space: 0,
            used_memory: 0
        }
    }
}

impl ProcessSortableReminders for ProcessWrapperPFF {
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
        self.desired_space = space.max(1);
    }
}

impl VirtualMemory for PageFaultFrequency {
    fn emulate(&self, processes: Vec<Process>) -> Vec<usize> {
        let mut memory = HashSet::new();
        let mut memory_to_take = 0;
        let mut lru = Lru::new();
        let mut page_to_process = Vec::new();
        let mut ans = vec![0; processes.len()];
        let mut processes: Vec<ProcessWrapperPFF> = processes.into_iter()
            .map(ProcessWrapperPFF::from)
            .collect();
        ReminderSorter::new(self.memory_size, processes.len())
            .set_frame_counts(
                processes.iter()
                    .map(|v| v.get_page_count())
                    .sum(),
                &mut processes
            );
        let mut done = 0;
        let mut i = 0;
        let mut abstract_timer = 0;
        while done < processes.len() {
            if let Some(accesses) = processes[i].process.next() {
                for access in accesses {
                    // Page to Process update
                    match page_to_process.len() <= access {
                        true => page_to_process.resize(access + 1, i),
                        false => page_to_process[access] = i,
                    }
                    // LRU update
                    lru.use_val(access, abstract_timer);
                    processes[i].lru.use_val(access, abstract_timer);
                    abstract_timer += 1;

                    // Check if in memory
                    if !memory.contains(&access) {
                        processes[i].page_faults_list.push_back(true);
                        processes[i].page_faults_count += 1;
                        match (
                            processes[i].desired_space > processes[i].used_memory,
                            memory.len() < self.memory_size,
                            memory_to_take != 0,
                            processes[i].used_memory != 0
                        ) {
                            // Want more and there is free space
                            (true, true, _, _) => {
                                memory.insert(access);
                                processes[i].used_memory += 1;
                                ans[i] += 1;
                            }
                            // Want more, and there is memory to take
                            (true, false, true, _) => {
                                let page_to_remove = lru.lru_iter().iter()
                                    .find(|v| {
                                        let process_id = page_to_process[*v.get_val()];
                                        processes[process_id].used_memory > 1 && processes[process_id].used_memory > processes[process_id].desired_space
                                    }).unwrap().clone();
                                let lru_page = *page_to_remove.get_val();
                                let process_id = page_to_process[lru_page];
                                processes[process_id].used_memory -= 1;
                                processes[process_id].lru.remove_val(lru_page);
                                memory.remove(page_to_remove.get_val());
                                lru.remove_lru(&page_to_remove);
                                memory_to_take -= 1;

                                processes[i].used_memory += 1;
                                memory.insert(access);
                                ans[i] += 1;
                            }
                            // Don't want more, has some or Want more, but there is no more free/to take but it has some
                            (false, _, _, true) | (true, false, false, true) => {
                                let lru_page = processes[i].lru.pop_lru().unwrap();
                                lru.remove_val(lru_page);
                                #[cfg(not(debug_assertions))]
                                memory.remove(&lru_page);
                                #[cfg(debug_assertions)]
                                if !memory.remove(&lru_page) {
                                    panic!("Memory was not released")
                                }
                                memory.insert(access);
                                ans[i] += 1;
                            }
                            // No memory available
                            (true, false, false, false) => {
                                #[cfg(debug_assertions)]
                                panic!("No memory");
                                #[cfg(not(debug_assertions))]
                                continue;
                            }
                            // No memory and no desire
                            (false, _, _, false) => {
                                panic!("No memory and no desire for it (PFF)")
                            }
                        }
                        
                    }
                    else {
                        processes[i].page_faults_list.push_back(false);
                    }

                    if processes[i].page_faults_list.len() > self.period {
                        processes[i].page_faults_count -= processes[i].page_faults_list.pop_front().unwrap() as usize;
                    }

                    if processes[i].page_faults_count > self.maximum_amount_of_page_faults {
                        processes[i].desired_space += 1;
                        if processes[i].desired_space <= processes[i].used_memory {
                            memory_to_take -= 1;
                        }
                    }
                    if processes[i].page_faults_count < self.minimal_amount_of_page_faults && processes[i].desired_space > 1 {
                        processes[i].desired_space -= 1;
                        if processes[i].desired_space < processes[i].used_memory {
                            memory_to_take += 1;
                        }
                    }
                }

                if processes[i].is_empty() {
                    done += 1;
                    memory_to_take -= processes[i].used_memory
                                        .saturating_sub(processes[i].desired_space);
                    processes[i].process.get_used_pages()
                        .iter()
                        .for_each(|v| {
                            memory.remove(v);
                            lru.remove_val(*v);
                        });
                    processes[i].desired_space = 0;
                    processes[i].used_memory = 0;
                }
            }

            i = (i + 1) % processes.len();
            #[cfg(debug_assertions)]
            if memory.len() > self.memory_size {
                panic!("Memory overflow (pff) - used memory: {}, available memory: {}", memory.len(), self.memory_size);
            }
        }
        ans
    }
}

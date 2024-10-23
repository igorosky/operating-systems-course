use std::collections::{BTreeSet, HashMap, HashSet, LinkedList};

use crate::{lru::Lru, process::Process, reminder_sorter::{ProcessSortableReminders, ReminderSorter}};

use super::VirtualMemory;

#[derive(Debug, Clone, Hash)]
pub(crate) struct WorkingSetSize {
    memory_size: usize,
    period: usize,
}

impl WorkingSetSize {
    #[inline]
    pub(crate) fn new(
        memory_size: usize,
        period: usize,
    ) -> Self {
        Self {
            memory_size,
            period,
        }
    } 
}

struct ProcessWrapperWSS {
    process: Process,
    lru: Lru<usize>,
    pages_list: LinkedList<usize>,
    pages_counts: HashMap<usize, usize>,
    desired_space: usize,
    used_memory: usize,
    is_halted: bool,
}

impl From<Process> for ProcessWrapperWSS {
    fn from(process: Process) -> Self {
        ProcessWrapperWSS {
            process,
            lru: Lru::new(),
            pages_list: LinkedList::new(),
            pages_counts: HashMap::new(),
            desired_space: 0,
            used_memory: 0,
            is_halted: false,
        }
    }
}

impl ProcessSortableReminders for ProcessWrapperWSS {
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

impl VirtualMemory for WorkingSetSize {
    fn emulate(&self, processes: Vec<Process>) -> Vec<usize> {
        let mut memory = HashSet::new();
        let mut memory_to_take = 0;
        let mut lru = Lru::new();
        let mut recently_halted_process = None;
        let mut processes_memory_desire: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut page_to_process = Vec::new();
        let mut ans = vec![0; processes.len()];
        let mut processes: Vec<ProcessWrapperWSS> = processes.into_iter()
            .map(ProcessWrapperWSS::from)
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
            if processes[i].is_empty() {
                i = (i + 1) % processes.len();
                continue;
            }
            if processes[i].is_halted {
                if processes[i].used_memory + memory_to_take + self.memory_size - memory.len() < processes[i].desired_space {
                    i = (i + 1) % processes.len();
                    continue;
                }
                processes[i].is_halted = false;
                processes[i].lru
                    .lru_iter()
                    .iter()
                    .for_each(|v| lru.use_val(*v.get_val(), v.get_time()));
                memory_to_take += processes[i].used_memory
                                    .saturating_sub(processes[i].desired_space);
                if recently_halted_process == Some(i) {
                    recently_halted_process = None;
                }
            }
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
                        match (
                            processes[i].desired_space > processes[i].used_memory,
                            memory.len() < self.memory_size,
                            memory_to_take != 0,
                            processes[i].used_memory != 0,
                            recently_halted_process.is_some(),
                            processes_memory_desire.len() > 1
                        ) {
                            // Want more and there is free space
                            (true, true, _, _, _, _) => {
                                memory.insert(access);
                                processes[i].used_memory += 1;
                                ans[i] += 1;
                            }
                            // Want more, and there is recently halted process
                            (true, false, _, _, true, _) => {
                                let halted_process_uses_no_memory = {
                                    let process_id = recently_halted_process.unwrap();
                                    #[cfg(debug_assertions)]
                                    assert!(recently_halted_process.is_some());
                                    processes[process_id].used_memory -= 1;
                                    let lru_page = processes[process_id].lru.pop_lru().unwrap();
                                    lru.remove_val(lru_page);
                                    memory.remove(&lru_page);
                                    processes[process_id].used_memory == 0
                                };
                                if halted_process_uses_no_memory {
                                    recently_halted_process = None;
                                }
                                memory.insert(access);
                                processes[i].used_memory += 1;
                                ans[i] += 1;
                            }
                            // Want more, and there is memory to take
                            (true, false, true, _, false, _) => {
                                let page_to_remove = lru.lru_iter().iter()
                                    .find(|v| {
                                        let process_id = page_to_process[*v.get_val()];
                                        processes[process_id].used_memory > 1 && processes[process_id].used_memory > processes[process_id].desired_space
                                    }).unwrap().clone();
                                let process_id = page_to_process[*page_to_remove.get_val()];
                                processes[process_id].used_memory -= 1;
                                processes[process_id].lru.remove_val(*page_to_remove.get_val());
                                #[cfg(not(debug_assertions))]
                                memory.remove(page_to_remove.get_val());
                                #[cfg(debug_assertions)]
                                if !memory.remove(page_to_remove.get_val()) {
                                    panic!("No memory were cleared (wss)");
                                }
                                lru.remove_lru(&page_to_remove);
                                memory_to_take -= 1;

                                processes[i].used_memory += 1;
                                memory.insert(access);
                                ans[i] += 1;
                            }
                            // Want more, but there is no more free/to take and there is process to halt
                            (true, false, false, _, false, true) => {
                                let (v, mut process_id) = processes_memory_desire.pop_last().unwrap();
                                if process_id == i {
                                    let tmp = processes_memory_desire.pop_last().unwrap().1;
                                    processes_memory_desire.insert((v, process_id));
                                    process_id = tmp;
                                }
                                memory_to_take -= processes[process_id].used_memory
                                                    .saturating_sub(processes[process_id].desired_space);
                                processes[process_id].is_halted = true;
                                processes[process_id].used_memory -= 1;
                                let lru_page = processes[process_id].lru.pop_lru().unwrap();
                                lru.remove_val(lru_page);
                                #[cfg(not(debug_assertions))]
                                memory.remove(&lru_page);
                                #[cfg(debug_assertions)]
                                if !memory.remove(&lru_page) {
                                    panic!("No memory were cleared (wss)");
                                }
                                if processes[process_id].used_memory != 0 {
                                    recently_halted_process = Some(process_id);
                                    processes[i].lru
                                        .lru_iter()
                                        .iter()
                                        .for_each(|v| lru.remove_lru(v));
                                }
                                memory.insert(access);
                                processes[i].used_memory += 1;
                                ans[i] += 1;
                            }
                            // Don't want more, has some or want some but there is no process to halt
                            (false, _, _, true, _, _) | (true, false, false, _, false, false) => {
                                let lru_page = processes[i].lru.pop_lru().unwrap();
                                #[cfg(not(debug_assertions))]
                                memory.remove(&lru_page);
                                #[cfg(debug_assertions)]
                                if !memory.remove(&lru_page) {
                                    panic!("No memory were cleared (wss)");
                                }
                                lru.remove_val(lru_page);
                                memory.insert(access);
                                ans[i] += 1;
                            }
                            // No memory and no desire
                            (false, _, _, false, _, _) => {
                                panic!("No memory and no desire for it (WSS)");
                            }
                        }
                        
                    }
                    processes[i].pages_list.push_back(access);
                    {
                        let current_count = *processes[i].pages_counts.get(&access).unwrap_or(&0);
                        processes[i].pages_counts.insert(access, current_count + 1);
                    }
                    if processes[i].pages_list.len() > self.period {
                        let old_page = processes[i].pages_list.pop_front().unwrap();
                        let is_zero = {
                            let b = processes[i].pages_counts.get_mut(&old_page).unwrap();
                            *b -= 1;
                            *b == 0
                        };
                        if is_zero {
                            processes[i].pages_counts.remove(&old_page);
                        }
                    }

                    if processes[i].pages_counts.len() > processes[i].desired_space {
                        if processes[i].desired_space < processes[i].used_memory {
                            memory_to_take -= (processes[i].pages_counts.len() - processes[i].desired_space)
                                                .min(processes[i].used_memory - processes[i].desired_space);
                        }
                        processes_memory_desire.remove(&(processes[i].desired_space, i));
                        processes[i].desired_space = processes[i].pages_counts.len();
                        if processes[i].used_memory != 0 {
                            processes_memory_desire.insert((processes[i].desired_space, i));
                        }
                    }
                    if processes[i].pages_counts.len() < processes[i].desired_space {
                        if processes[i].pages_counts.len() < processes[i].used_memory {
                            memory_to_take += (processes[i].used_memory - processes[i].pages_counts.len())
                                                .min(processes[i].desired_space - processes[i].pages_counts.len());
                        }
                        processes_memory_desire.remove(&(processes[i].desired_space, i));
                        processes[i].desired_space = processes[i].pages_counts.len();
                        if processes[i].used_memory != 0 {
                            processes_memory_desire.insert((processes[i].desired_space, i));
                        }
                    }
                }

                if processes[i].is_empty() {
                    done += 1;
                    processes_memory_desire.remove(&(processes[i].desired_space, i));
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
                panic!("Memory overflow (wss) - used memory: {}, available memory: {}", memory.len(), self.memory_size);
            }
        }
        ans
    }
}

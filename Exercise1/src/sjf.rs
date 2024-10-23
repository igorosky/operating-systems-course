use std::cmp::Reverse;

use crate::cpu_access_manager::{self, CpuAccessManager, Process};

#[derive(Debug, Clone)]
pub struct SJF {
    queue: std::collections::BinaryHeap<Reverse<Process>>,
    current_process: Option<Process>,
    working_time: u32,
    waiting_time: u32,
    total_working_time: u32,
    total_waiting_time: u32,
    next_id: u32,
    current_time: u32,
    statistics: Vec<cpu_access_manager::ProcessStatistics>,
    total_required_time: u32,
}

impl SJF {
    #[inline]
    pub fn new() -> Self {
        Self { queue: std::collections::BinaryHeap::new(), current_process: None, working_time: 0, waiting_time: 0, total_working_time: 0, total_waiting_time: 0, next_id: 0, current_time: 0, statistics: Vec::new(), total_required_time: 0 }
    }
}

impl CpuAccessManager for SJF {
    fn add_process(&mut self, time: u32, lifetime: Option<u32>) -> u32 {
        if time == 0 {
            panic!("Process can't have duration time of 0");
        }
        self.queue.push(Reverse(Process::new(self.next_id, self.current_time, time, lifetime)));
        let ans = self.next_id;
        self.next_id += 1;
        if self.current_process.is_none() {
            self.current_process = self.queue.pop().map(|v| v.0);
        }
        self.total_required_time += time;
        ans
    }

    #[inline]
    fn is_working(&self) -> bool {
        self.current_process.is_some()
    }

    #[inline]
    fn simulate_one_tick(&mut self) {
        self.simulate_n_ticks(1)
    }

    fn simulate_n_ticks(&mut self, mut n: u32) {
        while n != 0 {
            if let Some(mut current) = self.current_process.take() {
                let worked_time = current.work_for(self.current_time, n, current.get_initial_time() == current.get_time_left());
                n -= worked_time;
                self.current_time += worked_time;
                self.total_required_time -= worked_time;
                if current.is_finished() {
                    self.statistics.push(current.finalize());
                    self.current_process = self.queue.pop().map(|v| v.0);
                }
                else {
                    self.current_process = Some(current);
                }
            }
            else {
                self.waiting_time = n;
                self.working_time = 0;
                self.total_waiting_time += self.waiting_time;
                return;
            }
        }
    }

    #[inline]
    fn get_total_working_time(&self) -> u32 {
        self.total_working_time
    }

    #[inline]
    fn get_total_waiting_time(&self) -> u32 {
        self.total_waiting_time
    }

    #[inline]
    fn get_working_time(&self) -> u32 {
        self.working_time
    }

    #[inline]
    fn get_waiting_time(&self) -> u32 {
        self.waiting_time
    }

    #[inline]
    fn simulate_till_end_of_every_process(&mut self) {
        self.simulate_n_ticks(self.total_required_time)
    }

    #[inline]
    fn finalize(mut self) -> Vec<cpu_access_manager::ProcessStatistics> {
        self.simulate_till_end_of_every_process();
        self.statistics
    }
}

#[derive(Debug, Clone)]
pub struct SJFWithPreemption {
    queue: std::collections::BinaryHeap<Reverse<Process>>,
    working_time: u32,
    waiting_time: u32,
    total_working_time: u32,
    total_waiting_time: u32,
    next_id: u32,
    current_time: u32,
    statistics: Vec<cpu_access_manager::ProcessStatistics>,
    total_required_time: u32,
    current_id: Option<u32>,
}

impl SJFWithPreemption {
    #[inline]
    pub fn new() -> Self {
        Self { queue: std::collections::BinaryHeap::new(), working_time: 0, waiting_time: 0, total_working_time: 0, total_waiting_time: 0, next_id: 0, current_time: 0, statistics: Vec::new(), total_required_time: 0, current_id: None }
    }
}

impl CpuAccessManager for SJFWithPreemption {
    fn add_process(&mut self, time: u32, lifetime: Option<u32>) -> u32 {
        if time == 0 {
            panic!("Process can't have duration time of 0");
        }
        self.queue.push(Reverse(Process::new(self.next_id, self.current_time, time, lifetime)));
        let ans = self.next_id;
        self.next_id += 1;
        self.total_required_time += time;
        ans
    }

    #[inline]
    fn is_working(&self) -> bool {
        !self.queue.is_empty()
    }

    #[inline]
    fn simulate_one_tick(&mut self) {
        self.simulate_n_ticks(1)
    }

    fn simulate_n_ticks(&mut self, mut n: u32) {
        while n != 0 {
            if let Some(Reverse(mut current)) = self.queue.pop() {
                let worked_time = current.work_for(self.current_time, n, match self.current_id.is_none() || self.current_id.unwrap() != current.get_id() {
                    true => {
                        self.current_id = Some(current.get_id());
                        true
                    },
                    false => false,
                });
                n -= worked_time;
                self.current_time += worked_time;
                self.total_required_time -= worked_time;
                if current.is_finished() {
                    self.statistics.push(current.finalize());
                }
                else {
                    self.queue.push(Reverse(current));
                }
            }
            else {
                self.waiting_time = n;
                self.working_time = 0;
                self.total_waiting_time += self.waiting_time;
                return;
            }
        }
    }

    #[inline]
    fn get_total_working_time(&self) -> u32 {
        self.total_working_time
    }

    #[inline]
    fn get_total_waiting_time(&self) -> u32 {
        self.total_waiting_time
    }

    #[inline]
    fn get_working_time(&self) -> u32 {
        self.working_time
    }

    #[inline]
    fn get_waiting_time(&self) -> u32 {
        self.waiting_time
    }

    #[inline]
    fn simulate_till_end_of_every_process(&mut self) {
        self.simulate_n_ticks(self.total_required_time)
    }

    #[inline]
    fn finalize(mut self) -> Vec<cpu_access_manager::ProcessStatistics> {
        self.simulate_till_end_of_every_process();
        self.statistics
    }
}


use crate::cpu_access_manager::{self, CpuAccessManager, Process};
use crate::loop_list::LoopListIter;

#[derive(Debug)]
pub struct Rotating {
    quantum_time: u32,
    time_left_for_current_process: u32,
    queue: LoopListIter<Process>,
    working_time: u32,
    waiting_time: u32,
    total_working_time: u32,
    total_waiting_time: u32,
    next_id: u32,
    current_time: u32,
    statistics: Vec<cpu_access_manager::ProcessStatistics>,
    total_required_time: u32,
}

impl Rotating {
    #[inline]
    pub fn new(quantum_time: u32) -> Self {
        Self { quantum_time, time_left_for_current_process: quantum_time, queue: LoopListIter::new(), working_time: 0, waiting_time: 0, total_working_time: 0, total_waiting_time: 0, next_id: 0, current_time: 0, statistics: Vec::new(), total_required_time: 0 }
    }
}

impl CpuAccessManager for Rotating {
    fn add_process(&mut self, time: u32, lifetime: Option<u32>) -> u32 {
        if time == 0 {
            panic!("Process can't have duration time of 0");
        }
        self.queue.add(Process::new(self.next_id, self.current_time, time, lifetime));
        let ans = self.next_id;
        self.next_id += 1;
        self.total_required_time += time;
        ans
    }

    fn is_working(&self) -> bool {
        !self.queue.is_empty()
    }

    fn simulate_one_tick(&mut self) {
        self.simulate_n_ticks(1)
    }

    fn simulate_n_ticks(&mut self, mut n: u32) {
        while n != 0 {
            if let Some(node) = self.queue.get() {
                let time_for_process = n.min(self.time_left_for_current_process);
                let worked_time = node.borrow_mut().get_mut().work_for(self.current_time, time_for_process, time_for_process == self.quantum_time);
                n -= worked_time;
                self.time_left_for_current_process -= worked_time;
                self.total_required_time -= worked_time;
                self.current_time += worked_time;
                if node.borrow().get().is_finished() {
                    drop(node);
                    self.statistics.push(self.queue.erase().unwrap().finalize());
                    self.time_left_for_current_process = self.quantum_time;
                    self.queue.next();
                }
                if self.time_left_for_current_process == 0 {   
                    self.time_left_for_current_process = self.quantum_time;
                    self.queue.next();
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
        self.simulate_n_ticks(self.total_required_time);
    }

    #[inline]
    fn finalize(mut self) -> Vec<cpu_access_manager::ProcessStatistics> {
        self.simulate_till_end_of_every_process();
        self.statistics
    }
}

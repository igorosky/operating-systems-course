use std::{cell::RefCell, rc::Rc};

use crate::{disk_access_manager::DiskAccessManager, drive::Drive, real_time_handler::RealTimeHandler, task::Task};

#[derive(Debug)]
pub struct SCAN<R> where R: RealTimeHandler {
    next_id: usize,
    tasks_list: Vec<Rc<RefCell<Task>>>,
    drive: Rc<RefCell<Drive>>,
    statistics: Vec<Task>,
    real_time_handler: R,
    going_right: bool,
}

impl<R> DiskAccessManager for SCAN<R> where R: RealTimeHandler {
    fn add_task(&mut self, position: usize, realtime: Option<usize>) {
        let is_real_time = realtime.is_some();
        let task = Rc::new(RefCell::new(Task::new(self.next_id, position, self.drive.borrow().get_current_time(), realtime)));
        self.drive.borrow_mut().add_task(task.clone());
        if is_real_time {
            self.real_time_handler.add_task(task);
        }
        else {
            self.tasks_list.push(task);
        }
    }

    fn simulate_n_ticks(&mut self, mut n: usize) {
        let (simulated_ticks, vec) = self.real_time_handler.simulate_n_ticks(n);
        self.add_vec_to_statistics(vec);
        n -= simulated_ticks;
        let mut borrow = self.drive.borrow_mut();
        while n != 0 {
            if self.going_right {
                let target = borrow.get_position().checked_add(n).unwrap_or(usize::MAX).min(borrow.len());
                n -= borrow.go_to_position(target);
                self.going_right = borrow.get_position() != borrow.len();
            }
            else {
                let target = borrow.get_position().checked_sub(n).unwrap_or(1);
                n -= borrow.go_to_position(target);
                self.going_right = borrow.get_position() == 1;
            }
        }
    }

    fn finalize(mut self) -> Vec<Task> {
        let vec = self.real_time_handler.finalize();
        self.add_vec_to_statistics(vec);
        self.add_waiting_to_statistics();
        if !self.tasks_list.is_empty() {
            let (min, max) = {
                let mut min = usize::MAX;
                let mut max = 0;
                for pos in self.tasks_list.iter().map(|v| v.borrow().get_position()) {
                    min = min.min(pos);
                    max = max.max(pos);
                }
                (min, max)
            };
            if self.going_right {
                if max > self.drive.borrow().get_position() {
                    self.drive.borrow_mut().go_to_position(max);
                    self.add_waiting_to_statistics();
                    if self.tasks_list.is_empty() {
                        return self.statistics;
                    }
                }
                let len = self.drive.borrow().len();
                self.drive.borrow_mut().go_to_position(len);
                self.drive.borrow_mut().go_to_position(min);
            }
            else {
                if min < self.drive.borrow().get_position() {
                    self.drive.borrow_mut().go_to_position(min);
                    self.add_waiting_to_statistics();
                    if self.tasks_list.is_empty() {
                        return self.statistics;
                    }
                }
                self.drive.borrow_mut().go_to_position(1);
                self.drive.borrow_mut().go_to_position(max);
            }
            self.add_waiting_to_statistics();
        }
        self.statistics
    }
}

impl<R> SCAN<R> where R: RealTimeHandler {
    #[inline]
    pub fn new(drive: Rc<RefCell<Drive>>) -> Self {
        Self { next_id: 0, tasks_list: Vec::new(), drive: drive.clone(), statistics: Vec::new(), real_time_handler: R::new(drive), going_right: true }
    }

    #[inline]
    fn add_to_statistics(&mut self, task: Rc<RefCell<Task>>) {
        match Rc::try_unwrap(task) {
            Ok(val) => self.statistics.push(val.into_inner()),
            _ => panic!("Unexpected behavior"),
        }
    }

    #[inline]
    fn add_vec_to_statistics(&mut self, vec: Vec<Rc<RefCell<Task>>>) {
        self.statistics.reserve(vec.len());
        vec.into_iter().for_each(|task| self.add_to_statistics(task));
    }

    #[inline]
    fn add_waiting_to_statistics(&mut self) {
        let mut i = 0;
        while i < self.tasks_list.len() {
            if self.tasks_list[i].borrow().is_done() {
                let task = self.tasks_list.swap_remove(i);
                self.add_to_statistics(task);
                continue;
            }
            i += 1;
        }
    }
}

use std::{cell::RefCell, rc::Rc};

use crate::{disk_access_manager::DiskAccessManager, drive::Drive, real_time_handler::RealTimeHandler, task::Task};

#[derive(Debug)]
pub struct SSTF<R> where R: RealTimeHandler {
    next_id: usize,
    tasks_list: Vec<Rc<RefCell<Task>>>,
    drive: Rc<RefCell<Drive>>,
    statistics: Vec<Task>,
    real_time_handler: R,
}

impl<R> DiskAccessManager for SSTF<R> where R: RealTimeHandler {
    fn add_task(&mut self, position: usize, realtime: Option<usize>) {
        let is_real_time = realtime.is_some();
        let task = Rc::new(RefCell::new(Task::new(self.next_id, position, self.drive.borrow().get_current_time(), realtime)));
        self.drive.borrow_mut().add_task(task.clone());
        if is_real_time {
            self.real_time_handler.add_task(task);
        }
        else {
            self.tasks_list.insert(match self.tasks_list.binary_search(&task) {
                Ok(v) | Err(v)=> v,
            }, task);
        }
    }

    fn simulate_n_ticks(&mut self, mut n: usize) {
        let (simulated_ticks, vec) = self.real_time_handler.simulate_n_ticks(n);
        n -= simulated_ticks;
        self.add_vec_to_statistics(vec);
        while n != 0 {
            if !self.tasks_list.is_empty() {
                let current_index = self.find_closest();
                let current = self.tasks_list[current_index].clone();
                let dest = current.borrow().get_position();
                let cur_pos = self.drive.borrow().get_position();
                if n < cur_pos.abs_diff(dest) {
                    n -= self.drive.borrow_mut().go_to_position(match cur_pos > dest {
                        true => cur_pos - n,
                        false => cur_pos + n,
                    });
                }
                else {
                    n -= self.drive.borrow_mut().go_to_position(dest);
                    self.tasks_list.remove(current_index);
                    self.add_to_statistics(current);
                }
            }
            else {
                self.drive.borrow_mut().wait_for(n);
                break;
            }
        }
    }

    fn finalize(mut self) -> Vec<Task> {
        let vec = self.real_time_handler.finalize();
        self.add_vec_to_statistics(vec);
        while !self.tasks_list.is_empty() {
            let next = self.find_closest();
            if self.tasks_list[next].borrow().is_done() {
                let current = self.tasks_list.remove(next);
                self.add_to_statistics(current);
                continue;
            }
            let pos = self.tasks_list[next].borrow().get_position();
            self.drive.borrow_mut().go_to_position(pos);
            let current = self.tasks_list.remove(next);
            self.add_to_statistics(current);
        }
        self.statistics
    }
}

impl<R> SSTF<R> where R: RealTimeHandler {
    #[inline]
    pub fn new(drive: Rc<RefCell<Drive>>) -> Self {
        Self { next_id: 0, tasks_list: Vec::new(), drive: drive.clone(), statistics: Vec::new(), real_time_handler: R::new(drive) }
    }

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

    fn find_closest(&self) -> usize {
        let mut p = 0;
        let mut q = self.tasks_list.len();
        while p + q < q {
            let s = (p + q) / 2;
            let val = self.tasks_list[s].borrow().get_position();
            match val.cmp(&self.drive.borrow().get_position()) {
                std::cmp::Ordering::Less => p = s,
                std::cmp::Ordering::Greater => q = s,
                std::cmp::Ordering::Equal => return s,
            }
        }
        if q < self.tasks_list.len() {
            match (self.drive.borrow().get_position() - self.tasks_list[p].borrow().get_position()) <= (self.tasks_list[q].borrow().get_position() - self.drive.borrow().get_position()) {
                true => p,
                false => q,
            }
        }
        else {
            p
        }
    }
}

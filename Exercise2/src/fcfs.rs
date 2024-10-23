use std::{cell::RefCell, collections::LinkedList, rc::Rc};

use crate::{disk_access_manager::DiskAccessManager, drive::Drive, real_time_handler::RealTimeHandler, task::Task};

#[derive(Debug)]
pub struct FCFS<R> where R: RealTimeHandler {
    next_id: usize,
    tasks_list: LinkedList<Rc<RefCell<Task>>>,
    drive: Rc<RefCell<Drive>>,
    statistics: Vec<Task>,
    real_time_handler: R,
}

impl<R> DiskAccessManager for FCFS<R> where R: RealTimeHandler {
    fn add_task(&mut self, position: usize, realtime: Option<usize>) {
        let is_real_time = realtime.is_some();
        let task = Rc::new(RefCell::new(Task::new(self.next_id, position, self.drive.borrow().get_current_time(), realtime)));
        self.drive.borrow_mut().add_task(task.clone());
        if is_real_time {
            self.real_time_handler.add_task(task);
        }
        else {
            self.tasks_list.push_back(task);
        }
    }

    fn simulate_n_ticks(&mut self, mut n: usize) {
        let (simulated_ticks, vec) = self.real_time_handler.simulate_n_ticks(n);
        self.add_vec_to_statistics(vec);
        n -= simulated_ticks;
        while n != 0 {
            if let Some(current) = self.tasks_list.pop_front() {
                if current.borrow().is_done() {
                    self.add_to_statistics(current);
                    continue;
                }
                let mut dest = current.borrow().get_position();
                let diff = dest.abs_diff(self.drive.borrow().get_position());
                if diff > n {
                    if self.drive.borrow().get_position() > dest {
                        dest = self.drive.borrow().get_position() - n;
                    }
                    else {
                        dest = self.drive.borrow().get_position() + n;
                    }
                    n -= self.drive.borrow_mut().go_to_position(dest);
                    self.tasks_list.push_front(current);
                }
                else {
                    n -= self.drive.borrow_mut().go_to_position(dest);
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
        while let Some(current) = self.tasks_list.pop_front() {
            if current.borrow().is_done() {
                self.add_to_statistics(current);
                continue;
            }
            let pos = current.borrow().get_position();
            self.drive.borrow_mut().go_to_position(pos);
            self.add_to_statistics(current);
        }
        self.statistics
    }
}

impl<R> FCFS<R> where R: RealTimeHandler {
    #[inline]
    pub fn new(drive: Rc<RefCell<Drive>>) -> Self {
        Self { next_id: 0, tasks_list: LinkedList::new(), drive: drive.clone(), statistics: Vec::new(), real_time_handler: R::new(drive) }
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
}

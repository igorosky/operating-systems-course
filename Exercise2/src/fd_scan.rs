use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

use crate::{drive::Drive, edf::TaskWrapper, real_time_handler::RealTimeHandler, task::Task};

#[derive(Debug, Clone)]
pub struct FDSCAN {
    tasks: BTreeSet<TaskWrapper>,
    drive: Rc<RefCell<Drive>>,
}

impl RealTimeHandler for FDSCAN {
    #[inline]
    fn new(drive: Rc<RefCell<Drive>>) -> Self {
        Self { tasks: BTreeSet::new(), drive }
    }

    fn add_task(&mut self, task: Rc<RefCell<Task>>) -> bool {
        self.tasks.insert(TaskWrapper::new(task))
    }

    fn is_any_real_time(&self) -> bool {
        self.tasks.iter().filter(|v| self.is_possible(v.as_ref())).any(|v| !v.as_ref().borrow().is_done())
    }

    fn simulate_n_ticks(&mut self, mut n: usize) -> (usize, Vec<Rc<RefCell<Task>>>) {
        let mut simulated_ticks = 0;
        let mut statistics = Vec::new();
        while n != 0 && !self.tasks.is_empty() {
            let current = self.tasks.first().unwrap().get();
            if current.borrow().is_done() {
                statistics.push(self.tasks.pop_first().unwrap().into());
                continue;
            }
            if !self.is_possible(&current) {
                current.borrow_mut().set_starved();
                self.drive.borrow_mut().remove_task(&current);
                statistics.push(self.tasks.pop_first().unwrap().into());
                continue;
            }
            let mut pos = current.borrow().get_position();
            if pos.abs_diff(self.drive.borrow().get_position()) > n {
                if self.drive.borrow().get_position() > pos {
                    pos = self.drive.borrow().get_position() - n;
                }
                else {
                    pos = self.drive.borrow().get_position() + n;
                }
                let m = self.drive.borrow_mut().go_to_position(pos);
                simulated_ticks += m;
                n -= m;
            }
            else {
                let m = self.drive.borrow_mut().go_to_position(pos);
                simulated_ticks += m;
                n -= m;
                statistics.push(self.tasks.pop_first().unwrap().into());
            }
        }
        (simulated_ticks, statistics)
    }

    fn finalize(&mut self) -> Vec<Rc<RefCell<Task>>> {
        let mut statistics = Vec::new();
        while let Some(TaskWrapper(current)) = self.tasks.pop_first() {
            if current.borrow().is_done() {
                statistics.push(current);
                continue;
            }
            if !self.is_possible(&current) {
                current.borrow_mut().set_starved();
                self.drive.borrow_mut().remove_task(&current);
                statistics.push(current);
                continue;
            }
            let pos = current.borrow().get_position();
            self.drive.borrow_mut().go_to_position(pos);
        }
        statistics
    }
}

impl FDSCAN {
    #[inline]
    fn is_possible(&self, task: &Rc<RefCell<Task>>) -> bool {
        let time_required = task.borrow().get_position().abs_diff(self.drive.borrow().get_position());
        time_required <= self.drive.borrow().get_current_time()
    }
}

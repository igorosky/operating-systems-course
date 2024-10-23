use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

use crate::{drive::Drive, real_time_handler::RealTimeHandler, task::Task};

#[derive(Debug, Clone)]
pub struct TaskWrapper (pub Rc<RefCell<Task>>);

impl TaskWrapper {
    pub fn new(val: Rc<RefCell<Task>>) -> Self {
        Self(val)
    }

    pub fn get(&self) -> Rc<RefCell<Task>> {
        self.0.clone()
    }
}

impl AsRef<Rc<RefCell<Task>>> for TaskWrapper {
    fn as_ref(&self) -> &Rc<RefCell<Task>> {
        &self.0
    }
}

impl Into<Rc<RefCell<Task>>> for TaskWrapper {
    fn into(self) -> Rc<RefCell<Task>> {
        self.0
    }
}

impl From<Rc<RefCell<Task>>> for TaskWrapper {
    fn from(value: Rc<RefCell<Task>>) -> Self {
        Self(value)
    }
}

impl PartialEq for TaskWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.borrow().get_id() == other.0.borrow().get_id()
    }
}

impl Eq for TaskWrapper {
    
}

impl PartialOrd for TaskWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.borrow().get_realtime().unwrap().partial_cmp(&other.0.borrow().get_realtime().unwrap()) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.0.borrow().get_id().partial_cmp(&other.0.borrow().get_id()) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        Some(std::cmp::Ordering::Equal)
    }
}

impl Ord for TaskWrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct EDF {
    tasks: BTreeSet<TaskWrapper>,
    drive: Rc<RefCell<Drive>>
}

impl RealTimeHandler for EDF {
    #[inline]
    fn new(drive: Rc<RefCell<Drive>>) -> Self {
        Self { tasks: BTreeSet::new(), drive }
    }

    fn add_task(&mut self, task: Rc<RefCell<Task>>) -> bool {
        self.tasks.insert(TaskWrapper::new(task))
    }

    fn is_any_real_time(&self) -> bool {
        self.tasks.iter().any(|v| !v.as_ref().borrow().is_done())
    }

    fn simulate_n_ticks(&mut self, mut n: usize) -> (usize, Vec<Rc<RefCell<Task>>>) {
        let mut simulated_ticks = 0;
        let mut statistics = Vec::new();
        while n != 0 && !self.tasks.is_empty() {
            let current = self.tasks.first().unwrap().get();
            if current.borrow().is_done() {
                statistics.push(self.tasks.pop_first().unwrap().get());
                continue;
            }
            if current.borrow().get_position().abs_diff(self.drive.borrow().get_position()) > n {
                let dst = match current.borrow().get_position() > self.drive.borrow().get_position() {
                    true => self.drive.borrow().get_position() + n,
                    false => self.drive.borrow().get_position() - n,
                };
                let m = self.drive.borrow_mut().go_to_position_skipping(dst);
                n -= m;
                simulated_ticks += m;
            }
            else {
                let pos = current.borrow().get_position();
                let m = self.drive.borrow_mut().go_to_position_skipping(pos);
                simulated_ticks += m;
                n -= m;
                statistics.push(self.tasks.pop_first().unwrap().get());
            }
        }
        (simulated_ticks, statistics)
    }
    
    fn finalize(&mut self) -> Vec<Rc<RefCell<Task>>> {
        let mut ans = Vec::with_capacity(self.tasks.len());
        while let Some(TaskWrapper(current)) = self.tasks.pop_first() {
            if current.borrow().is_done() {
                ans.push(current);
                continue;
            }
            let pos = current.borrow().get_position();
            self.drive.borrow_mut().go_to_position_skipping(pos);
            ans.push(current);
        }
        ans
    }
}

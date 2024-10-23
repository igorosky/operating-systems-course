use std::{cell::RefCell, rc::Rc};

use crate::{drive::Drive, task::Task};

pub trait RealTimeHandler {
    fn new(drive: Rc<RefCell<Drive>>) -> Self;
    fn add_task(&mut self, task: Rc<RefCell<Task>>) -> bool;    
    fn is_any_real_time(&self) -> bool;
    fn simulate_n_ticks(&mut self, n: usize) -> (usize, Vec<Rc<RefCell<Task>>>);
    fn finalize(&mut self) -> Vec<Rc<RefCell<Task>>>;
}

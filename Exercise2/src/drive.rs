use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

use crate::task::Task;

#[derive(Debug, Clone)]
pub struct Drive {
    len: usize,
    position: usize,
    move_count: usize,
    roll_count: usize,
    tasks: BTreeSet<Rc<RefCell<Task>>>,
    current_time: usize,
}

impl Drive {
    #[inline]
    fn is_in_range(&self, position: usize) {
        if position > self.len {
            panic!("Out of range");
        }
    }

    #[inline]
    pub fn get_move_count(&self) -> usize {
        self.move_count
    }
    
    #[inline]
    pub fn new(len: usize) -> Self {
        Self::new_on_pos(len, 1)
    }

    #[inline]
    pub fn new_on_pos(len: usize, position: usize) -> Self {
        if position > len {
            panic!("Out of range");
        }
        Self { len, position, move_count: 0, roll_count: 0, tasks: BTreeSet::new(), current_time: 0 }
    }
    
    #[inline]
    pub fn get_position(&self) -> usize {
        self.position
    }
    
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn remove_tasks_in_range(&mut self, range: std::ops::RangeInclusive<usize>) {
        self.tasks.retain(|p| {
            let mut borrow = p.borrow_mut();
            let pos = borrow.get_position();
            if range.contains(&pos) {
                borrow.finalize(self.current_time + self.position.abs_diff(pos));
                false
            }
            else {
                true
            }
        });
    }

    #[inline]
    pub fn go_to_position(&mut self, position: usize) -> usize {
        self.is_in_range(position);
        let distance = self.position.abs_diff(position);
        self.remove_tasks_in_range(self.position.min(position)..=self.position.max(position));
        self.move_count += distance;
        self.current_time += distance;
        self.position = position;
        distance
    }

    #[inline]
    pub fn roll(&mut self) -> bool {
        if self.position != self.len {
            return false;
        }
        self.roll_count += 1;
        
        self.tasks.retain(|p| {
            let mut borrow = p.borrow_mut();
            let pos = borrow.get_position();
            if pos == 1 || pos == self.len {
                borrow.finalize(self.current_time + ((pos == 1) as usize));
                false
            }
            else {
                true
            }
        });
        
        self.current_time += 1;
        true
    }

    #[inline]
    pub fn get_roll_count(&self) -> usize {
        self.roll_count
    }

    #[inline]
    pub fn get_current_time(&self) -> usize {
        self.current_time
    }

    #[inline]
    pub fn add_task(&mut self, task: Rc<RefCell<Task>>) -> bool {
        if task.borrow().get_position() == self.position {
            task.borrow_mut().finalize(self.current_time);
            true
        }
        else {
            self.tasks.insert(task)
        }
    }

    #[inline]
    pub fn remove_task(&mut self, task: &Rc<RefCell<Task>>) -> bool {
        self.tasks.remove(task)
    }

    #[inline]
    pub fn wait_for(&mut self, time: usize) {
        self.current_time += time
    }

    #[inline]
    pub fn go_to_position_skipping(&mut self, position: usize) -> usize {
        self.is_in_range(position);
        let distance = self.position.abs_diff(position);
        self.remove_tasks_in_range(position..=position);
        self.position = position;
        self.move_count += distance;
        self.current_time += distance;
        distance
    }
}

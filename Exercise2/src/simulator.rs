use std::{cell::RefCell, rc::Rc};

use crate::{c_scan::CSCAN, disk_access_manager::DiskAccessManager, drive::Drive, edf::EDF, fcfs::FCFS, fd_scan::FDSCAN, scan::SCAN, sstf::SSTF, task::State};

#[derive(Debug, Clone)]
pub struct Tasks {
    process_list: std::collections::LinkedList<(usize, usize, Option<usize>)>,
}

impl Tasks {
    // #[inline]
    // fn next_duration(&mut self) -> Option<u32> {
    //     match self.process_list.pop_front() {
    //         Some((_, next)) => Some(next),
    //         None => None,
    //     }
    // }

    // #[inline]
    // fn next_wait(&self) -> Option<u32> {
    //     match self.process_list.front() {
    //         Some((next, _)) => Some(*next),
    //         None => None,
    //     }
    // }

    #[inline]
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.process_list.len()
    }

    #[inline]
    fn next(&mut self) -> Option<(usize, usize, Option<usize>)> {
        self.process_list.pop_front()
    }
}

impl From<Vec<(usize, usize, Option<usize>)>> for Tasks {
    fn from(value: Vec<(usize, usize, Option<usize>)>) -> Self {
        let mut list = std::collections::LinkedList::new();
        for val in value {
            list.push_back(val);
        }
        Self { process_list: list }
    }
}

struct Simulation<T> where T : DiskAccessManager {
    disk: T,
    tasks_list: Tasks,
}

#[derive(Debug, Clone)]
pub struct SimulationStatistics {
    task_count: usize,
    moves_count: usize,
    rolls_count: usize,
    count_of_realtime_tasks: usize,
    count_of_successful_realtime_tasks: usize,
    total_realtime_tasks_waiting_time: usize,
    total_non_realtime_tasks_waiting_time: usize,
}

impl SimulationStatistics {
    #[inline]
    pub fn get_task_count(&self) -> usize {
        self.task_count
    }

    #[inline]
    pub fn get_moves_count(&self) -> usize {
        self.moves_count
    }

    #[inline]
    pub fn get_rolls_count(&self) -> usize {
        self.rolls_count
    }

    #[inline]
    pub fn get_count_of_realtime_tasks(&self) -> usize {
        self.count_of_realtime_tasks
    }

    #[inline]
    pub fn get_count_of_successful_realtime_tasks(&self) -> usize {
        self.count_of_successful_realtime_tasks
    }

    #[inline]
    pub fn get_total_realtime_tasks_waiting_time(&self) -> usize {
        self.total_realtime_tasks_waiting_time
    }

    #[inline]
    pub fn get_total_non_realtime_tasks_waiting_time(&self) -> usize {
        self.total_non_realtime_tasks_waiting_time
    }

}

impl<T> Simulation<T> where T : DiskAccessManager {
    pub fn new(disk: T, tasks_list: Tasks) -> Self {
        Self { disk, tasks_list }
    }

    pub fn simulate(mut self, drive: Rc<RefCell<Drive>>) -> SimulationStatistics {
        while let Some((time_to_wait, duration, lifetime)) = self.tasks_list.next() {
            self.disk.simulate_n_ticks(time_to_wait);
            let _ = self.disk.add_task(duration, lifetime);
        }
        println!("Finalizing");
        let task_statistics = self.disk.finalize();
        let task_count = task_statistics.len();
        let mut count_of_realtime_tasks = 0;
        let mut count_of_successful_realtime_tasks = 0;
        let mut total_realtime_tasks_waiting_time = 0;
        let mut total_non_realtime_tasks_waiting_time = 0;
        for task in task_statistics {
            if task.is_realtime() {
                count_of_realtime_tasks += 1;
                if let State::SUCCESSFUL(end) = task.get_state() {
                    total_realtime_tasks_waiting_time += end - task.get_creation_time();
                    count_of_successful_realtime_tasks += 1;
                }
            }
            else if let State::SUCCESSFUL(end) = task.get_state() {
                total_non_realtime_tasks_waiting_time += end - task.get_creation_time();
            }
        }
        SimulationStatistics { task_count, moves_count: drive.borrow().get_move_count(), rolls_count: drive.borrow().get_roll_count(), count_of_realtime_tasks, count_of_successful_realtime_tasks, total_realtime_tasks_waiting_time, total_non_realtime_tasks_waiting_time }
    }
}

pub fn simulate_every(tasks_list: Tasks, disk_size: usize) -> Vec<(String, SimulationStatistics)> {
    let mut ans = Vec::with_capacity(8);
    
    let drive = Rc::new(RefCell::new(Drive::new(disk_size)));
    ans.push(("FCFS-EDF".to_owned(), Simulation::new(FCFS::<EDF>::new(drive.clone()), tasks_list.clone()).simulate(drive)));
    println!("1/8");
    
    let drive = Rc::new(RefCell::new(Drive::new(disk_size)));
    ans.push(("SSTF-EDF".to_owned(), Simulation::new(SSTF::<EDF>::new(drive.clone()), tasks_list.clone()).simulate(drive)));
    println!("2/8");
    
    let drive = Rc::new(RefCell::new(Drive::new(disk_size)));
    ans.push(("SCAN-EDF".to_owned(), Simulation::new(SCAN::<EDF>::new(drive.clone()), tasks_list.clone()).simulate(drive)));
    println!("3/8");
    
    let drive = Rc::new(RefCell::new(Drive::new(disk_size)));
    ans.push(("CSCAN-EDF".to_owned(), Simulation::new(CSCAN::<EDF>::new(drive.clone()), tasks_list.clone()).simulate(drive)));
    println!("4/8");
    
    

    let drive = Rc::new(RefCell::new(Drive::new(disk_size)));
    ans.push(("FCFS-FDSCAN".to_owned(), Simulation::new(FCFS::<FDSCAN>::new(drive.clone()), tasks_list.clone()).simulate(drive)));
    println!("5/8");
    
    let drive = Rc::new(RefCell::new(Drive::new(disk_size)));
    ans.push(("SSTF-FDSCAN".to_owned(), Simulation::new(SSTF::<FDSCAN>::new(drive.clone()), tasks_list.clone()).simulate(drive)));
    println!("6/8");
    
    let drive = Rc::new(RefCell::new(Drive::new(disk_size)));
    ans.push(("SCAN-FDSCAN".to_owned(), Simulation::new(SCAN::<FDSCAN>::new(drive.clone()), tasks_list.clone()).simulate(drive)));
    println!("7/8");
    
    let drive = Rc::new(RefCell::new(Drive::new(disk_size)));
    ans.push(("CSCAN-FDSCAN".to_owned(), Simulation::new(CSCAN::<FDSCAN>::new(drive.clone()), tasks_list).simulate(drive)));
    println!("8/8");
    
    ans
}

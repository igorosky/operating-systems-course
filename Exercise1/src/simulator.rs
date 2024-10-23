use crate::{cpu_access_manager::CpuAccessManager, fcfs, rotating, sjf};

#[derive(Debug, Clone)]
pub struct Processes {
    process_list: std::collections::LinkedList<(u32, u32, Option<u32>)>,
}

impl Processes {
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
    fn next(&mut self) -> Option<(u32, u32, Option<u32>)> {
        self.process_list.pop_front()
    }
}

impl From<Vec<(u32, u32, Option<u32>)>> for Processes {
    fn from(value: Vec<(u32, u32, Option<u32>)>) -> Self {
        let mut list = std::collections::LinkedList::new();
        for val in value {
            list.push_back(val);
        }
        Self { process_list: list }
    }
}

struct Simulation<T> where T : CpuAccessManager {
    cpu: T,
    process_list: Processes,
}

pub struct SimulationStatistics {
    average_waiting_time: f64,
    processes_count: usize,
    longest_waiting_time: u32,
    average_call_count: f64,
    average_partial_waiting_time: f64,
    processes_with_lifetime: usize,
    finished_processes: usize,
    successful_processes: usize,
    average_call_count_of_successful_processes: f64,
}

impl SimulationStatistics {
    #[inline]
    pub fn get_average_waiting_time(&self) -> f64 {
        self.average_waiting_time
    }

    #[inline]
    pub fn get_processes_count(&self) -> usize {
        self.processes_count
    }

    #[inline]
    pub fn get_longest_waiting_time(&self) -> u32 {
        self.longest_waiting_time
    }

    #[inline]
    pub fn get_average_call_count(&self) -> f64 {
        self.average_call_count
    }

    #[inline]
    pub fn get_average_partial_waiting_time(&self) -> f64 {
        self.average_partial_waiting_time
    }

    #[inline]
    pub fn get_lifetime_processes_count(&self) -> usize {
        self.processes_with_lifetime
    }

    #[inline]
    #[allow(dead_code)]
    pub fn get_finished_processes_count(&self) -> usize {
        self.finished_processes
    }

    #[inline]
    pub fn get_successful_processes_count(&self) -> usize {
        self.successful_processes
    }

    #[inline]
    pub fn get_average_call_count_of_successful_processes(&self) ->f64 {
        self.average_call_count_of_successful_processes
    }
}

impl<T> Simulation<T> where T : CpuAccessManager {
    pub fn new(cpu: T, process_list: Processes) -> Self {
        Self { cpu, process_list }
    }

    pub fn simulate(mut self) -> SimulationStatistics {
        while let Some((time_to_wait, duration, lifetime)) = self.process_list.next() {
            self.cpu.simulate_n_ticks(time_to_wait);
            let _ = self.cpu.add_process(duration, lifetime);
        }
        let mut average_waiting_time = 0f64;
        let processes_statistics = self.cpu.finalize();
        let processes_count = processes_statistics.len();
        let mut longest_waiting_time = 0;
        let mut average_call_count = 0f64;
        let mut average_partial_waiting_time = 0_f64;
        let mut processes_with_lifetime = 0;
        let mut finished_processes = 0;
        let mut successful_processes = 0;
        let mut call_count_of_successful_processes = 0f64;
        for process in processes_statistics {
            let waiting_time = process.get_completion_time().unwrap() + process.get_time_left() - process.get_creation_time() - process.get_initial_time();
            average_waiting_time += waiting_time as f64;
            longest_waiting_time = longest_waiting_time.max(waiting_time);
            average_call_count += process.get_call_count() as f64;
            average_partial_waiting_time += process.get_partial_waiting_time();
            processes_with_lifetime += process.has_lifetime() as usize;
            if let Some(successful) = process.is_successful() {
                finished_processes += 1;
                successful_processes += successful as usize;
                call_count_of_successful_processes += (process.get_call_count() as f64) * (successful as i32 as f64);
            }
            // println!("Id: {}", process.get_id());
            // println!("Creation time: {}", process.get_creation_time());
            // println!("End time: {:?}", process.get_completion_time());
            // println!("Initial time: {}", process.get_initial_time());
            // println!("Time left: {}", process.get_time_left());
            // println!("Count: {}", process.get_count());
        }
        let avg_div = processes_count.max(1) as f64;
        SimulationStatistics { average_waiting_time: average_waiting_time / avg_div, processes_count, longest_waiting_time, average_call_count: average_call_count / avg_div, average_partial_waiting_time: average_partial_waiting_time / avg_div, processes_with_lifetime, finished_processes, successful_processes, average_call_count_of_successful_processes: call_count_of_successful_processes / (successful_processes.max(1) as f64) }
    }
}

pub fn simulate_every(process_list: Processes, quant: u32) -> Vec<(String, SimulationStatistics)> {
    if quant == 0 {
        panic!("Quant cannot be 0");
    }
    let mut ans = Vec::with_capacity(4);
    ans.push(("FCFS".to_owned(), Simulation::new(fcfs::FCFS::new(), process_list.clone()).simulate()));
    ans.push(("SJF".to_owned(), Simulation::new(sjf::SJF::new(), process_list.clone()).simulate()));
    ans.push(("SJF with preemption".to_owned(), Simulation::new(sjf::SJFWithPreemption::new(), process_list.clone()).simulate()));
    ans.push(("Rotating".to_owned(), Simulation::new(rotating::Rotating::new(quant), process_list).simulate()));
    ans
}


#[derive(Clone, Debug)]
pub struct Process {
    id: u32,
    initial_time: u32,
    time_left: u32,
    creation_time: u32,
    completion_time: Option<u32>,
    call_count: usize,
    partial_waiting_time: f64,
    last_time_with_access: u32,
    lifetime: Option<u32>,
    finished_state: Option<bool>, // None - unfinished, true - successful, false - unsuccessful
}

#[derive(Debug, Clone)]
pub struct ProcessStatistics {
    process: Process,
}

impl ProcessStatistics {
    #[inline]
    #[allow(dead_code)]
    pub fn get_id(&self) -> u32 {
        self.process.get_id()
    }

    #[inline]
    pub fn get_initial_time(&self) -> u32 {
        self.process.get_initial_time()
    }

    #[inline]
    #[allow(dead_code)]
    pub fn get_time_left(&self) -> u32 {
        self.process.get_time_left()
    }

    #[inline]
    pub fn get_creation_time(&self) -> u32 {
        self.process.get_creation_time()
    }

    #[inline]
    pub fn get_completion_time(&self) -> Option<u32> {
        self.process.get_completion_time()
    }

    #[inline]
    pub fn get_call_count(&self) -> usize {
        self.process.get_call_count()
    }
    
    #[inline]
    pub fn get_partial_waiting_time(&self) -> f64 {
        self.process.get_partial_waiting_time()
    }

    #[inline]
    #[allow(dead_code)]
    pub fn is_finished(&self) -> bool {
        self.process.is_finished()
    }

    #[inline]
    pub fn is_successful(&self) -> Option<bool> {
        self.process.is_successful()
    }

    #[inline]
    pub fn has_lifetime(&self) -> bool {
        self.process.has_lifetime()
    }

    #[inline]
    #[allow(dead_code)]
    pub fn get_lifetime(&self) -> Option<u32> {
        self.process.get_lifetime()
    }
}

impl Process {
    pub fn new(id: u32, creation_time: u32, time: u32, lifetime: Option<u32>) -> Self {
        Self { id, initial_time: time, time_left: time, creation_time, completion_time: None, call_count: 0, partial_waiting_time: 0f64, last_time_with_access: creation_time, lifetime, finished_state: None }
    }

    // Returns utilized time
    pub fn work_for(&mut self, current_time: u32, time: u32, is_new_call: bool) -> u32 {
        if let Some(lifetime) = self.lifetime {
            if self.creation_time + lifetime < current_time + time {
                let ans = match self.creation_time + lifetime > current_time {
                    true => self.work_for(current_time, self.creation_time + lifetime - current_time, is_new_call),
                    false => 0,
                };
                self.completion_time = Some(current_time + ans);
                self.finished_state = Some(false);
                return ans;
            }
        }
        if time == 0 {
            return time;
        }
        if is_new_call {
            self.partial_waiting_time += (current_time - self.last_time_with_access) as f64 * (self.time_left / self.initial_time) as f64;
            self.call_count += 1;
        }
        if time >= self.time_left {
            let time_left = self.time_left;
            self.completion_time = Some(current_time + self.time_left);
            self.last_time_with_access = current_time + time_left;
            self.time_left = 0;
            self.finished_state = Some(true);
            return time_left;
        }
        self.time_left -= time;
        self.last_time_with_access = current_time + time;
        time
    }

    #[inline]
    pub fn get_id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn get_initial_time(&self) -> u32 {
        self.initial_time
    }

    #[inline]
    pub fn get_time_left(&self) -> u32 {
        self.time_left
    }

    #[inline]
    pub fn get_creation_time(&self) -> u32 {
        self.creation_time
    }

    #[inline]
    pub fn get_completion_time(&self) -> Option<u32> {
        self.completion_time
    }

    #[inline]
    pub fn finalize(self) -> ProcessStatistics {
        ProcessStatistics { process: self }
    }

    #[inline]
    pub fn get_call_count(&self) -> usize {
        self.call_count
    }

    #[inline]
    pub fn get_partial_waiting_time(&self) -> f64 {
        self.partial_waiting_time
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.finished_state.is_some()
    }

    #[inline]
    pub fn is_successful(&self) -> Option<bool> {
        self.finished_state
    }

    #[inline]
    pub fn has_lifetime(&self) -> bool {
        self.lifetime.is_some()
    }

    #[inline]
    pub fn get_lifetime(&self) -> Option<u32> {
        self.lifetime
    }
}

impl PartialEq for Process {
    fn eq(&self, other: &Self) -> bool {
        self.get_time_left() == other.get_time_left()
    }
}

impl PartialOrd for Process {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_time_left().partial_cmp(&other.get_time_left())
    }
}

impl Eq for Process {
    
}

impl Ord for Process {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_time_left().cmp(&other.get_time_left())
    }
}

pub trait CpuAccessManager {
    fn add_process(&mut self, time: u32, lifetime: Option<u32>) -> u32;
    fn is_working(&self) -> bool;
    fn simulate_one_tick(&mut self) {
        self.simulate_n_ticks(1)
    }
    fn simulate_n_ticks(&mut self, n: u32);
    fn simulate_till_end_of_every_process(&mut self);
    fn get_total_working_time(&self) -> u32;
    fn get_total_waiting_time(&self) -> u32;
    fn get_working_time(&self) -> u32;
    fn get_waiting_time(&self) -> u32;
    fn finalize(self) -> Vec<ProcessStatistics>;
}

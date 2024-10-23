use crate::{approximated_lru_virtual_memory_simulator::ApproximatedLRUVirtualMemorySimulator, fifo_virtual_memory_simulator::FIFOVirtualMemorySimulator, lru_virtual_memory_simulator::LRUVirtualMemorySimulator, opt_virtual_memory_simulator::OPTVirtualMemorySimulator, rand_virtual_memory_simulator::RandVirtualMemorySimulator, virtual_memory_simulator::VirtualMemorySimulator};

#[derive(Debug, Clone)]
pub struct MemoryAccessRequests {
    requests: Vec<usize>
}

impl From<Vec<usize>> for MemoryAccessRequests {
    fn from(requests: Vec<usize>) -> Self {
        Self { requests }
    }
}

#[derive(Debug, Clone)]
pub struct SimulationStatistics {
    // task_count: usize,
    // memory_size: usize,
    page_faults_count: usize,
}

impl SimulationStatistics {
    #[inline]
    pub fn get_page_faults_count(&self) -> usize {
        self.page_faults_count
    }

    // #[inline]
    // pub fn get_requests_count(&self) -> usize {
    //     self.task_count
    // }

    // #[inline]
    // pub fn get_memory_size(&self) -> usize {
    //     self.memory_size
    // }
}

#[inline]
pub fn simulate<T: VirtualMemorySimulator>(accesses_list: MemoryAccessRequests, memory_size: usize) -> SimulationStatistics {
    SimulationStatistics { /*task_count: accesses_list.requests.len(), memory_size,*/ page_faults_count: T::create_and_simulate(memory_size, accesses_list.requests) }
}

pub fn simulate_every(accesses_list: MemoryAccessRequests, memory_size: usize) -> Vec<(String, SimulationStatistics)> {
    let mut ans = Vec::with_capacity(5);

    ans.push(("FIFO".to_string(), simulate::<FIFOVirtualMemorySimulator>(accesses_list.clone(), memory_size)));
    ans.push(("OPT".to_string(), simulate::<OPTVirtualMemorySimulator>(accesses_list.clone(), memory_size)));
    ans.push(("LRU".to_string(), simulate::<LRUVirtualMemorySimulator>(accesses_list.clone(), memory_size)));
    ans.push(("AppxLRU".to_string(), simulate::<ApproximatedLRUVirtualMemorySimulator>(accesses_list.clone(), memory_size)));
    ans.push(("Rand".to_string(), simulate::<RandVirtualMemorySimulator>(accesses_list, memory_size)));
    
    ans
}

pub mod lazy_process_dispatcher;
pub mod ambitious_process_dispatcher;
pub mod helping_process_dispatcher;

use rand::Rng;

use crate::{process::Process, processor::ProcessorWrapper};

pub trait ProcessDispatcher: std::fmt::Debug + Sync {
    fn dispatch_new_process(&self, process: Process, processor_id: usize, network: &mut [ProcessorWrapper]);
    fn update(&self, processor_id: usize, network: &mut [ProcessorWrapper]);
}

pub(super) fn get_random_processor(network_len: usize, current_processor_id: usize) -> usize {
    let mut random = rand::thread_rng();
    let mut target_processor = random.gen_range(0..(network_len - 1));
    if target_processor >= current_processor_id {
        target_processor += 1;
    }
    target_processor
}

pub(super) fn try_give_process_away(
    asks_count: usize,
    max_load: f64,
    network: &mut [ProcessorWrapper],
    mut process: Process,
    processor_id: usize,
) -> Option<Process> {
    let network_len = network.len();
    if network_len < 2 {
        return Some(process);
    }
    let mut requests = 0;
    while requests < asks_count {
        let target_processor = &mut network[get_random_processor(
            network_len,
            processor_id
        )];
        let (target_processor_load, target_processor_free_resources) = target_processor.get_processor_load();
        if target_processor_load < max_load
        && process.required_resources() <= target_processor_free_resources {
            process.send();
            target_processor.processor_mut().spawn_process(process);
            return None;
        }
        requests += 1;
    }
    Some(process)
}

use std::collections::LinkedList;

use getset::CopyGetters;

use crate::{process::Process, process_dispatcher::try_give_process_away, processor::ProcessorWrapper};

use super::ProcessDispatcher;

#[derive(Debug, Clone, CopyGetters)]
pub struct AmbitiousProcessDispatcher {
    #[getset(get_copy = "pub")]
    max_self_load: f64,
    #[getset(get_copy = "pub")]
    max_other_load: f64,
    #[getset(get_copy = "pub")]
    requests_in_tick: usize,
}

impl AmbitiousProcessDispatcher {
    #[inline]
    pub fn new(max_self_load: f64, max_other_load: f64, requests_in_tick: usize) -> Self {
        Self { max_self_load, max_other_load, requests_in_tick }
    }
}

impl ProcessDispatcher for AmbitiousProcessDispatcher {
    fn dispatch_new_process(&self, process: Process, processor_id: usize, network: &mut [ProcessorWrapper]) {
        assert!(processor_id < network.len(), "Processor is not in network");
        let (load, available) = network[processor_id].self_get_processor_load();
        match load < self.max_self_load && available >= process.required_resources() {
            true => network[processor_id].processor_mut().spawn_process(process),
            false => 
                if let Some(process) = try_give_process_away(
                    self.requests_in_tick,
                    self.max_other_load,
                    network,
                    process,
                    processor_id
                ) {
                    network[processor_id].processes_queue_mut().push_back(process);
                }
        }
    }

    fn update(&self, processor_id: usize, network: &mut [ProcessorWrapper]) {
        assert!(processor_id < network.len(), "Processor is not in network");
        network[processor_id].processes_queue_mut()
            .iter_mut()
            .for_each(|process| process.wait());
        let mut new_queue = LinkedList::new();
        std::mem::swap(&mut new_queue, network[processor_id].processes_queue_mut());
        new_queue.into_iter().for_each(|process| self.dispatch_new_process(process, processor_id, network));
    }
}

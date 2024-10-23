use getset::CopyGetters;

use crate::{process::Process, processor::ProcessorWrapper};

use super::{try_give_process_away, ProcessDispatcher};

#[derive(Debug, Clone, CopyGetters)]
pub struct LazyProcessDispatcher {
    max_load: f64,
    asks_count: usize,
}

impl LazyProcessDispatcher {
    #[inline]
    pub fn new(max_load: f64, asks_count: usize) -> Self {
        Self { max_load, asks_count }
    }
}

impl ProcessDispatcher for LazyProcessDispatcher {
    fn dispatch_new_process(&self,
        process: Process,
        processor_id: usize,
        network: &mut [ProcessorWrapper],
    ) {
        assert!(processor_id < network.len(), "Processor is not in network");
        if let Some(process) = try_give_process_away(
            self.asks_count,
            self.max_load,
            network,
            process,
            processor_id
        ) {
            network[processor_id].processor_mut().spawn_process(process);
        }
    }

    #[inline]
    fn update(&self, _: usize, _: &mut [ProcessorWrapper]) {

    }
}

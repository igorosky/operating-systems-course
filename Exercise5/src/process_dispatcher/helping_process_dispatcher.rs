use getset::CopyGetters;

use crate::{process::Process, processor::ProcessorWrapper};

use super::{ambitious_process_dispatcher::AmbitiousProcessDispatcher, get_random_processor, ProcessDispatcher};

#[derive(Debug, Clone, CopyGetters)]
pub struct HelpingProcessDispatcher {
    ambitious_process_dispatcher: AmbitiousProcessDispatcher,
    #[getset(get_copy = "pub")]
    helping_threshold: f64,
    #[getset(get_copy = "pub")]
    other_helping_threshold: f64,
}

impl HelpingProcessDispatcher {
    pub fn new(max_self_load: f64,
        max_other_load: f64,
        requests_in_tick: usize,
        helping_threshold: f64,
        other_helping_threshold: f64
    ) -> Self {
        Self {
            ambitious_process_dispatcher: AmbitiousProcessDispatcher::new(
                max_self_load,
                max_other_load,
                requests_in_tick
            ),
            helping_threshold,
            other_helping_threshold
        }
    }
}

impl ProcessDispatcher for HelpingProcessDispatcher {
    #[inline]
    fn dispatch_new_process(&self, process: Process, processor_id: usize, network: &mut [ProcessorWrapper]) {
        self.ambitious_process_dispatcher.dispatch_new_process(process, processor_id, network)
    }

    fn update(&self, processor_id: usize, network: &mut [ProcessorWrapper]) {
        self.ambitious_process_dispatcher.update(processor_id, network);
        let (mut load, mut available) = network[processor_id].self_get_processor_load();
        let mut tries = 0;
        while load <= self.helping_threshold && tries < self.ambitious_process_dispatcher.requests_in_tick() {
            let random_processor = get_random_processor(
                network.len(),
                processor_id
            );
            let (random_processor_load, _) = network[random_processor].self_get_processor_load();
            if random_processor_load >= self.other_helping_threshold {
                let process = network[random_processor].give_process_away(available, self.helping_threshold);
                if let Some(process) = process {
                    network[processor_id].processor_mut().spawn_process(process);
                }
            }
            (load, available) = network[processor_id].self_get_processor_load();
            tries += 1;
        }
    }
}

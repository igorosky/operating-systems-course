use std::{collections::LinkedList, str::FromStr};

use getset::{CopyGetters, Getters};
use process::Process;
use process_dispatcher::{ambitious_process_dispatcher::AmbitiousProcessDispatcher, helping_process_dispatcher::HelpingProcessDispatcher, lazy_process_dispatcher::LazyProcessDispatcher, ProcessDispatcher};
use processor::{Processor, ProcessorStatistics, ProcessorWrapper};
use rand::Rng;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use statistics::Statistics;

mod process;
mod processor;
mod process_dispatcher;
mod statistics;

fn input_with_default<T, S>(prompt: S, default: T) -> T
    where
    T: Clone + ToString + FromStr,
    T::Err: ToString,
    S: Into<String> + Clone, {
    loop {
        match dialoguer::Input::<T>::new().with_prompt(prompt.clone()).default(default.clone()).interact() {
            Ok(ans) => return ans,
            Err(err) => println!("Dialoguer error: {}", err),
        }
    }
}

#[inline]
fn input_min_max_with_default<T, S>(prompt: S, default_min: T, default_max: T) -> (T, T)
    where
    T: Clone + ToString + FromStr + Ord,
    T::Err: ToString,
    S: Into<String> + Clone, 
{
    let prompt = prompt.into();
    let min = input_with_default(
        format!("Minimal {}", prompt.clone()),
        default_min
    );
    (
        min.clone(),
        input_with_default(
            format!("Maximal {}", prompt),
            default_max
        ).max(min)
    )
}

#[derive(Debug, Clone)]
struct ProcessesToDispatch {
    process: Process,
    time_gap: usize,
    processor_id: usize,
}

impl ProcessesToDispatch {
    #[inline]
    fn new(process: Process, time_gap: usize, processor_id: usize) -> Self {
        Self { process, time_gap, processor_id }
    }
}

fn simulate<D: ProcessDispatcher>(
    network: &mut [ProcessorWrapper],
    mut processes: LinkedList<ProcessesToDispatch>,
    dispatcher: D,
) -> (Vec<Vec<ProcessorStatistics>>, usize) {
    let mut processors_used_resources = vec![Vec::new(); network.len()];
    
    let mut timer = 0;
    let mut absolute_timer = 0;
    let mut anything_was_done = false;
    while !processes.is_empty() || anything_was_done {
        // Firstly dispatch processes
        let mut start_new_process = match processes.front() {
            Some(process) => process.time_gap <= timer,
            None => false,
        };
        while start_new_process {
            let ProcessesToDispatch{process, time_gap: _, processor_id} = processes.pop_front().unwrap();
            dispatcher.dispatch_new_process(process, processor_id, network);
            timer = 0;
            start_new_process = match processes.front() {
                Some(process) => process.time_gap <= timer,
                None => false,
            };
        }

        // Secondly Check Statistics
        processors_used_resources.par_iter_mut()
            .enumerate()
            .for_each(|(i, used_resources)|
                used_resources.push(network[i].get_statistics())
            );
        
        // Thirdly run one tick
        anything_was_done = network.par_iter_mut()
            .any(|processor| processor.work());

        // Lastly update dispatcher (actually used only in helping dispatcher)
        (0..network.len()).for_each(|processor_id| dispatcher.update(processor_id, network));

        timer += 1;
        absolute_timer += 1;
    }

    (processors_used_resources, absolute_timer)
}

#[derive(Debug, Clone, Default, Getters, CopyGetters)]
struct SimulationStatistics {
    #[getset(get = "pub")]
    sending_statistics: Option<Statistics<f64>>,
    #[getset(get = "pub")]
    waiting_statistics: Option<Statistics<f64>>,
    #[getset(get = "pub")]
    working_statistics: Option<Statistics<f64>>,
    #[getset(get = "pub")]
    requests_count_statistics: Option<Statistics<f64>>,
    #[getset(get = "pub")]
    processors_usage_statistics: Option<Statistics<f64>>,
    #[getset(get = "pub")]
    processes_count_statistics: Option<Statistics<f64>>,
    #[getset(get_copy = "pub")]
    total_time: usize,
}

fn simulate_and_get_statistics<D: ProcessDispatcher>(
    mut network: Vec<ProcessorWrapper>,
    processes: LinkedList<ProcessesToDispatch>,
    dispatcher: D,
) -> SimulationStatistics {
    let processes_len = processes.len();
    let (statistics_collected_during_simulation, total_time) = simulate(&mut network, processes, dispatcher);
    
    let mut sends = Vec::with_capacity(processes_len);
    let mut waiting_time = Vec::with_capacity(processes_len);
    let mut working_time = Vec::with_capacity(processes_len);
    let mut requests_count = Vec::with_capacity(network.len());
    let mut processors_usage = Vec::with_capacity(network.len().max(processes_len));
    let mut processes_count = Vec::with_capacity(network.len().max(processes_len));

    statistics_collected_during_simulation.into_par_iter()
        .enumerate()
        .map(|(i, s)| {
            let resources = network[i].processor().resources() as f64;
            s.into_par_iter()
                .map(|v| (v.used_resources() as f64 / resources, v.processes_count() as f64))
                .collect()
        })
        .collect::<Vec<Vec<_>>>()
        .into_iter()
        .for_each(|v| v.into_iter().for_each(|(usage, pc)| {
            processors_usage.push(usage);
            processes_count.push(pc);
        }));
    
    for processor in network {
        requests_count.push(processor.load_requests() as f64);
        for process in processor.finish_and_get_finished_processes() {
            sends.push(process.send_count() as f64);
            waiting_time.push(process.waiting_time() as f64);
            working_time.push(process.working_time() as f64);
        }
    }

    let mut sending_statistics = None;
    let mut waiting_statistics = None;
    let mut working_statistics = None;
    let mut requests_count_statistics = None;
    let mut processors_usage_statistics = None;
    let mut processes_count_statistics = None;

    rayon::scope(|s| {
        s.spawn(|_| sending_statistics = Statistics::par_statistics_of(&sends));
        s.spawn(|_| waiting_statistics = Statistics::par_statistics_of(&waiting_time));
        s.spawn(|_| working_statistics = Statistics::par_statistics_of(&working_time));
        s.spawn(|_| requests_count_statistics = Statistics::par_statistics_of(&requests_count));
        s.spawn(|_| processors_usage_statistics = Statistics::par_statistics_of(&processors_usage));
        s.spawn(|_| processes_count_statistics = Statistics::par_statistics_of(&processes_count));
    });

    SimulationStatistics {
        sending_statistics,
        waiting_statistics,
        working_statistics,
        requests_count_statistics,
        processors_usage_statistics,
        processes_count_statistics,
        total_time,
    }
}

fn print_results(results: SimulationStatistics) {
    println!("Total required time: {}", results.total_time());
    println!();
    println!("Sending statistics:");
    match results.sending_statistics {
        Some(val) => println!("{}", val),
        None => println!("None"),
    }
    println!();
    println!("Waiting statistics:");
    match results.waiting_statistics {
        Some(val) => println!("{}", val),
        None => println!("None"),
    }
    println!();
    println!("Working statistics:");
    match results.working_statistics {
        Some(val) => println!("{}", val),
        None => println!("None"),
    }
    println!();
    println!("Requests count statistics:");
    match results.requests_count_statistics {
        Some(val) => println!("{}", val),
        None => println!("None"),
    }
    println!();
    println!("Processors usage statistics:");
    match results.processors_usage_statistics {
        Some(val) => println!("{}", val),
        None => println!("None"),
    }
    println!();
    println!("Processes count statistics:");
    match results.processes_count_statistics {
        Some(val) => println!("{}", val),
        None => println!("None"),
    }
    println!();
}

fn run_simulation_and_display_results(
    network: Vec<ProcessorWrapper>,
    processes_to_dispatch: LinkedList<ProcessesToDispatch>,
) {
    let mut lazy_results = SimulationStatistics::default();
    let mut ambitious_results = SimulationStatistics::default();
    let mut helping_results = SimulationStatistics::default();

    rayon::scope(|s| {
        s.spawn(|_| {
            lazy_results = simulate_and_get_statistics(network.clone(), processes_to_dispatch.clone(), LazyProcessDispatcher::new(0.7, 30));
            println!("Lazy done");
        });
        s.spawn(|_| {
            ambitious_results = simulate_and_get_statistics(network.clone(), processes_to_dispatch.clone(), AmbitiousProcessDispatcher::new(0.7, 0.7, 30));
            println!("Ambitious done");
        });
        s.spawn(|_| {
            helping_results = simulate_and_get_statistics(network.clone(), processes_to_dispatch.clone(), HelpingProcessDispatcher::new(0.7, 0.7, 30, 0.2, 0.7));
            println!("Helping done");
        });
    });

    println!("\nLazy:");
    print_results(lazy_results);
    println!("Ambitious:");
    print_results(ambitious_results);
    println!("Helping:");
    print_results(helping_results);
}

fn random_test() {
    println!("Processors parameters:");
    let (processor_count_min, processor_count_max) = input_min_max_with_default("Processors count", 50, 200);
    let (processor_resources_min, processor_resources_max) = input_min_max_with_default("Processors resources", 100, 2000);

    println!("Processes parameters:");
    let (processes_count_min, processes_count_max) = input_min_max_with_default("Processes count", 100, 2000);
    let (mut processes_required_resources_min, mut processes_required_resources_max) = input_min_max_with_default("Processes required resources", 20, 2000);
    let (processes_required_time_min, processes_required_time_max) = input_min_max_with_default("Processes required time", 5, 100);
    let (processes_time_gap_min, processes_time_gap_max) = input_min_max_with_default("Processes time gap", 0, 35);

    let mut random = rand::thread_rng();
    let mut strongest_processor = 0;
    let network = Vec::from_iter(
        (0..random.gen_range(processor_count_min..=processor_count_max))
            .map(|_| {
                let resources = random.gen_range(processor_resources_min..=processor_resources_max);
                strongest_processor = strongest_processor.max(resources);
                ProcessorWrapper::new(Processor::new(resources))
            }
        ));
    
    processes_required_resources_max = processes_required_resources_max.min(strongest_processor);
    processes_required_resources_min = processes_required_resources_min.min(processes_required_resources_max);

    let processes_to_dispatch = LinkedList::from_iter(
        (0..random.gen_range(processes_count_min..=processes_count_max))
            .map(|i| ProcessesToDispatch::new(
                Process::new_random(
                    i,
                    processes_required_time_min..=processes_required_time_max,
                    processes_required_resources_min..=processes_required_resources_max
                ),
                random.gen_range(processes_time_gap_min..=processes_time_gap_max),
                random.gen_range(0..network.len())
        )));

    println!("Network size: {}", network.len());
    println!("Processes to dispatch size: {}", processes_to_dispatch.len());
    println!();
    run_simulation_and_display_results(network, processes_to_dispatch);
}

fn main() {
    loop {
        match dialoguer::Select::new()
            .items(&["Random test", "Exit"])
            .with_prompt("Select option")
            .interact()
            .expect("IO Error")
        {
            0 => random_test(),
            _ => break,
        }
    }
}

use std::fmt::{Debug, Display};
#[cfg(not(debug_assertions))]
use std::str::FromStr;

use process::Process;
use rand::Rng;
use virtual_memory::{equal_allocation::EqualAllocation, page_fault_frequency::PageFaultFrequency, proportional::Proportional, working_set_size::WorkingSetSize, VirtualMemory};
use num_format::{Locale, ToFormattedString};

mod virtual_memory;
mod process;
mod lru;
mod reminder_sorter;

#[derive(Debug, Clone)]
struct PFFParameters {
    min: usize,
    max: usize,
    period: usize,
}

#[derive(Debug, Clone)]
struct WSSParameters {
    period: usize,
}

#[derive(Debug, Clone)]
struct SimulationResults {
    name: String,
    sum: usize,
    mean: f64,
    median: usize,
    std_deviation: f64,
    variance: f64,
    min: usize,
    max: usize,
}

impl Display for SimulationResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
r#"Name: {}
Total: {}
Mean: {:.2}
Median: {}
Standard deviation: {:.2}
Variance: {:.2}
Min: {}
Max: {}
"#,
            self.name,
            self.sum.to_formatted_string(&Locale::fr),
            self.mean,
            self.median.to_formatted_string(&Locale::fr),
            self.std_deviation,
            self.variance,
            self.min.to_formatted_string(&Locale::fr),
            self.max.to_formatted_string(&Locale::fr)
        )
    }
}

fn simulate_one(name: String, vm: impl VirtualMemory, processes: Vec<Process>) -> SimulationResults {
    let results = vm.emulate(processes);
    let sum = results
        .iter()
        .sum::<usize>();
    let result_as_float = results
        .iter()
        .map(|v| *v as f64)
        .collect::<Vec<_>>();
    let mean = sum as f64 / results.len().max(1) as f64;
    SimulationResults {
        name,
        sum,
        mean,
        median: statistical::median(&results),
        std_deviation: statistical::standard_deviation(
            &result_as_float,
            Some(mean)),
        variance: statistical::variance(&result_as_float, Some(mean)),
        min: *results
            .iter()
            .min()
            .unwrap_or(&0),
        max: *results
            .iter()
            .max()
            .unwrap_or(&0),
    }
}

#[cfg(not(debug_assertions))]
fn run_simulation(processes: Vec<Process>, memory_size: usize, pff_parameters: PFFParameters, wss_parameters: WSSParameters) -> Vec<SimulationResults> {
    let mut handles = Vec::with_capacity(4);
    handles.push({
        let processes = processes.clone();
        std::thread::spawn(move ||
            simulate_one("EqualAllocation".to_string(), EqualAllocation::new(memory_size), processes)
        )
    });
    handles.push({
        let processes = processes.clone();
        std::thread::spawn(move ||
            simulate_one("Proportional".to_string(), Proportional::new(memory_size), processes)
        )
    });
    handles.push({
        let processes = processes.clone();
        std::thread::spawn(move ||
            simulate_one("PageFaultFrequency".to_string(), PageFaultFrequency::new(memory_size, pff_parameters.min, pff_parameters.max, pff_parameters.period), processes)
        )
    });
    handles.push({
        let processes = processes.clone();
        std::thread::spawn(move ||
            simulate_one("WorkingSetSize".to_string(), WorkingSetSize::new(memory_size, wss_parameters.period), processes)
        )
    });

    Vec::from_iter(
        handles
        .into_iter()
        .map(|v| v.join().unwrap())
    )
}

#[cfg(debug_assertions)]
#[inline]
fn run_simulation(processes: Vec<Process>, memory_size: usize, pff_parameters: PFFParameters, wss_parameters: WSSParameters) -> Vec<impl Display + Debug> {
    vec![
        simulate_one("EqualAllocation".to_string(), EqualAllocation::new(memory_size), processes.clone()),
        simulate_one("Proportional".to_string(), Proportional::new(memory_size), processes.clone()),
        simulate_one("PageFaultFrequency".to_string(), PageFaultFrequency::new(memory_size, pff_parameters.min, pff_parameters.max, pff_parameters.period), processes.clone()),
        simulate_one("WorkingSetSize".to_string(), WorkingSetSize::new(memory_size, wss_parameters.period), processes),
    ]
}

fn simulate(processes: Vec<Process>, memory_size: usize, pff_parameters: PFFParameters, wss_parameters: WSSParameters) {
    println!();
    println!("Processes count: {}", processes.len());
    println!("Memory size: {}", memory_size);
    println!("Amount of pages: {}",
        processes
            .iter()
            .map(|v| v.get_used_pages()
                .iter()
                .count()
            )
            .sum::<usize>()
            .to_formatted_string(&Locale::fr));
    println!();
    run_simulation(processes, memory_size, pff_parameters, wss_parameters)
        .into_iter()
        .for_each(|v| println!("{}", v));
}

#[cfg(not(debug_assertions))]
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

#[cfg(not(debug_assertions))]
fn random_test() {
    let memory_size_min = input_with_default("Minimal memory size", 100);
    let memory_size_max = input_with_default("Maximal memory size", 1_000)
        .max(memory_size_min);

    let processes_count_min = input_with_default("Minimal processes count", 100);
    let processes_count_max = input_with_default("Maximal processes count", 1_000)
        .max(processes_count_min);
    
    let total_len_min = input_with_default("Minimal request groups count", 100);
    let total_len_max = input_with_default("Maximal request groups count", 1_000)
        .max(total_len_min);
    
    let len_min = input_with_default("Minimal group len", 10);
    let len_max = input_with_default("Maximal group len", 20)
        .max(len_min);
    
    let pages_count_for_process_min = input_with_default("Minimal process page count", 100);
    let pages_count_for_process_max = input_with_default("Maximal process page count", 1000)
        .max(pages_count_for_process_min);
    
    let pff_min_min = input_with_default("PFF min min", 5)
        .max(1);
    let pff_min_max = input_with_default("PFF min max", 15)
        .max(pff_min_min);
    
    let pff_max_min = input_with_default("PFF max min", 15)
        .max(pff_min_max);
    let pff_max_max = input_with_default("PFF max max", 25)
        .max(pff_max_min);

    let pff_period_min = input_with_default("PFF period min", 10)
        .max(1);
    let pff_period_max = input_with_default("PFF period max", 30)
        .max(pff_period_min);

    let wss_period_min = input_with_default("WSS period min", 10)
        .max(1);
    let wss_period_max = input_with_default("WSS period max", 30)
        .max(wss_period_min);
    
    let mut random = rand::thread_rng();

    let processes_count = random.gen_range(processes_count_min..=processes_count_max);
    
    let mut pages_offset = 0;
    let processes = Vec::from_iter((0..processes_count).map(|_| {
        let pages_count = random.gen_range(pages_count_for_process_min..=pages_count_for_process_max);
        let ans = Process::new(
            total_len_min..=total_len_max,
            len_min..=len_max,
            pages_offset..(pages_offset + pages_count)
        );
        pages_offset += pages_count;
        ans
    }));

    let memory_size = random.gen_range(memory_size_min..=memory_size_max);
    
    simulate(
        processes,
        memory_size,
        PFFParameters {
            min: random.gen_range(pff_min_min..=pff_min_max),
            max: random.gen_range(pff_max_min..=pff_max_max),
            period: random.gen_range(pff_period_min..=pff_period_max)
        },
        WSSParameters { period: random.gen_range(wss_period_min..=wss_period_max) }
    );
}

#[cfg(debug_assertions)]
fn debug_test() {
    std::env::set_var("RUST_BACKTRACE", "full");
    let memory_size_min = 1000;
    let memory_size_max = 1_000;

    let processes_count_min = 10;
    let processes_count_max = 100;
    
    let total_len_min = 100;
    let total_len_max = 1_000;
    
    let len_min = 10;
    let len_max = 20;
    
    let pages_count_for_process_min = 10;
    let pages_count_for_process_max = 100;
    
    let mut random = rand::thread_rng();

    let processes_count = random.gen_range(processes_count_min..=processes_count_max);
    
    let mut pages_offset = 0;
    let processes = Vec::from_iter((0..processes_count).map(|_| {
        let pages_count = random.gen_range(pages_count_for_process_min..=pages_count_for_process_max);
        let ans = Process::new(
            total_len_min..=total_len_max,
            len_min..=len_max,
            pages_offset..(pages_offset + pages_count)
        );
        pages_offset += pages_count;
        ans
    }));

    let memory_size = random.gen_range(memory_size_min..=memory_size_max);
    
    simulate(processes, memory_size, PFFParameters { min: 5, max: 15, period: 10 }, WSSParameters { period: 10 });
}

fn main() {
    #[cfg(debug_assertions)]
    debug_test();
    #[cfg(not(debug_assertions))]
    loop {
        match dialoguer::Select::new()
            .items(&["Random Test", "Exit"])
            .with_prompt("Select option")
            .interact() {
            Ok(0) => random_test(),
            Ok(_) => break,
            Err(err) => println!("Error: {}", err),
        }
    }
}

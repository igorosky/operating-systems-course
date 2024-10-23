use std::str::FromStr;

use num_format::{ToFormattedString, Locale};
use rand::Rng;
use simulator::SimulationStatistics;

mod cpu_access_manager;
mod fcfs;
mod sjf;
mod rotating;
mod loop_list;
mod simulator;

extern crate rand;
extern crate dialoguer;
extern crate num_format;

fn input<T, S>(prompt: S) -> T
    where
    T: Clone + ToString + FromStr,
    T::Err: ToString,
    S: Into<String> + Clone, {
    loop {
        match dialoguer::Input::<T>::new().with_prompt(prompt.clone()).interact() {
            Ok(ans) => return ans,
            Err(err) => println!("Dialoguer error: {}", err),
        }
    }
}

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

fn print_results(results: Vec<(String, SimulationStatistics)>) {
    for (name, result) in results {
        println!("{}", name);
        println!("Processes count: {}", result.get_processes_count().to_formatted_string(&Locale::fr));
        println!("Average waiting time to complete: {:.2}", result.get_average_waiting_time());
        println!("Longest waiting time: {}", result.get_longest_waiting_time().to_formatted_string(&Locale::fr));
        println!("Average call count: {:.2}", result.get_average_call_count());
        println!("Average call count of successful processes: {:.2}", result.get_average_call_count_of_successful_processes());
        println!("Average partial waiting time: {:.2}", result.get_average_partial_waiting_time());
        println!("Processes with lifetime count: {}", result.get_lifetime_processes_count().to_formatted_string(&Locale::fr));
        println!("Processes ended successfully: {}", result.get_successful_processes_count().to_formatted_string(&Locale::fr));
        let successful_lifetime_processes = result.get_successful_processes_count() - (result.get_processes_count() - result.get_lifetime_processes_count());
        println!("Lifetime processes ended successfully: {}", successful_lifetime_processes.to_formatted_string(&Locale::fr));
        println!("Successful lifetime processes ratio: {:.2}", successful_lifetime_processes as f64 / result.get_lifetime_processes_count() as f64);
        println!("==================")
    }
}

fn random_test_menu(quant: u32) {
    let minimum_number_of_processes_in_test: usize = input_with_default("Minimum number of processes in test", 50);
    let maximum_number_of_processes_in_test: usize = input_with_default("Maximum number of processes in test", 1000).max(minimum_number_of_processes_in_test);
    let minimum_process_duration: u32 = input_with_default("Minimum process duration (cannot be 0, it will be incremented if so)", 1).max(1);
    let maximum_process_duration: u32 = input_with_default("Maximum process duration", 300);
    let minimum_time_between_new_processes: u32 = input_with_default("Minimum time between new processes", 0);
    let maximum_time_between_new_processes: u32 = input_with_default("Maximum time between new processes", 100).max(minimum_time_between_new_processes);
    let lifetime_probability: u32 = input_with_default("Life time probability per mil [0;1000]", 300);
    let minimum_lifetime = input_with_default("Minimum lifetime", 1);
    let maximum_lifetime = input_with_default("Maximum lifetime", 2000);

    print_results(simulator::simulate_every(simulator::Processes::from({
        let mut rng = rand::thread_rng();
        let process_count = rng.gen_range(minimum_number_of_processes_in_test..=maximum_number_of_processes_in_test);
        let mut processes_list = Vec::with_capacity(process_count);
        for _ in 0..process_count {
            processes_list.push((rng.gen_range(minimum_time_between_new_processes..=maximum_time_between_new_processes), rng.gen_range(minimum_process_duration..=maximum_process_duration), match rng.gen_range(0..1000).cmp(&lifetime_probability) {
                std::cmp::Ordering::Less => Some(rng.gen_range(minimum_lifetime..=maximum_lifetime)),
                _ => None,
            }));
        }
        processes_list
    }), quant));
}

fn manual_test(quant: u32) {
    let mut processes = Vec::with_capacity(input("Processes count"));
    for i in 1..=processes.capacity() {
        println!("Process {}.", i);
        processes.push((input("Time to start after previous process"), input("Process duration"), match dialoguer::Select::new().with_prompt("With probability").items(&["No", "Yes"]).interact() {
            Ok(1) => Some(input("Lifetime")),
            _ => None,
        }));
    }
    print_results(simulator::simulate_every(simulator::Processes::from(processes), quant));
}

fn main() {
    let mut quant = 5;
    loop {
        match dialoguer::Select::new()
            .items(&["Random tests", "Manual test", "Change quant", "Exit"])
            .with_prompt("Select option")
            .interact() {
            Ok(0) => random_test_menu(quant),
            Ok(1) => manual_test(quant),
            Ok(2) => quant = input_with_default("Quant time", quant).max(1),
            Ok(3) => break,
            Ok(_) | Err(_) => (),
        }
    }
    // let processes_lists = vec![simulator::Processes::from(vec![(0, 5), (0, 2), (0, 10), (0, 4)])];
    // for processes_list in processes_lists {
    //     simulator::simulate_every(processes_list);
    // }
}

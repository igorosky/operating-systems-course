use std::str::FromStr;

use num_format::{ToFormattedString, Locale};
use rand::Rng;

mod disk_access_manager;
mod drive;
mod task;
mod fcfs;
mod sstf;
mod real_time_handler;
mod edf;
mod scan;
mod c_scan;
mod fd_scan;
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

fn print_results(results: Vec<(String, simulator::SimulationStatistics)>) {
    for (name, result) in results {
        println!("{}", name);
        println!("Tasks count: {}", result.get_task_count().to_formatted_string(&Locale::fr));
        println!("Realtime tasks: {}", result.get_count_of_realtime_tasks().to_formatted_string(&Locale::fr));
        println!("Average waiting time: {:.2}", (result.get_total_non_realtime_tasks_waiting_time() + result.get_total_realtime_tasks_waiting_time()) as f64 / (result.get_task_count().max(1) as f64));
        println!("Average waiting time (non-realtime): {:.2}", result.get_total_non_realtime_tasks_waiting_time() as f64 / ((result.get_task_count() - result.get_count_of_realtime_tasks()).max(1) as f64));
        println!("Average waiting time (realtime): {:.2}", result.get_total_realtime_tasks_waiting_time() as f64 / (result.get_count_of_realtime_tasks().max(1) as f64));
        println!("Realtime tasks finished successfully: {}", result.get_count_of_successful_realtime_tasks().to_formatted_string(&Locale::fr));
        println!("Moves count: {}", result.get_moves_count().to_formatted_string(&Locale::fr));
        println!("Rolls count: {}", result.get_rolls_count().to_formatted_string(&Locale::fr));
        println!("==================")
    }
}

fn random_test_menu(disk_size: usize) {
    let minimum_number_of_tasks_in_test: usize = input_with_default("Minimum number of tasks in test", 50);
    let maximum_number_of_tasks_in_test: usize = input_with_default("Maximum number of tasks in test", 10000).max(minimum_number_of_tasks_in_test);
    let minimum_address: usize = input_with_default("Minimum address", 1).max(1);
    let maximum_address: usize = input_with_default("Maximum address", disk_size).max(minimum_address).min(disk_size);
    let minimum_time_between_new_tasks: usize = input_with_default("Minimum time between new tasks", 0);
    let maximum_time_between_new_tasks: usize = input_with_default("Maximum time between new tasks", 100).max(minimum_time_between_new_tasks);
    let realtime_probability: u32 = input_with_default("Realtime time probability per mil [0;1000]", 100);
    let minimum_realtime = input_with_default("Minimum realtime", 1);
    let maximum_realtime = input_with_default("Maximum realtime", 2000);

    print_results(simulator::simulate_every(simulator::Tasks::from({
        let mut rng = rand::thread_rng();
        let process_count = rng.gen_range(minimum_number_of_tasks_in_test..=maximum_number_of_tasks_in_test);
        let mut processes_list = Vec::with_capacity(process_count);
        for _ in 0..process_count {
            processes_list.push((rng.gen_range(minimum_time_between_new_tasks..=maximum_time_between_new_tasks), rng.gen_range(minimum_address..=maximum_address), match rng.gen_range(0..1000).cmp(&realtime_probability) {
                std::cmp::Ordering::Less => Some(rng.gen_range(minimum_realtime..=maximum_realtime)),
                _ => None,
            }));
        }
        processes_list
    }), disk_size));
}

fn manual_test(disk_size: usize) {
    let mut processes = Vec::with_capacity(input("Tasks count"));
    for i in 1..=processes.capacity() {
        println!("Process {}.", i);
        processes.push((input("Time to start after previous task"), input::<usize, &str>("Address").max(1).min(disk_size), match dialoguer::Select::new().with_prompt("With probability").items(&["No", "Yes"]).interact() {
            Ok(1) => Some(input("Lifetime")),
            _ => None,
        }));
    }
    print_results(simulator::simulate_every(simulator::Tasks::from(processes), disk_size));
}

fn main() {
    let mut disk_size = 1000;
    loop {
        match dialoguer::Select::new()
            .items(&["Random tests", "Manual test", "Change disk size", "Exit"])
            .with_prompt("Select option")
            .interact() {
            Ok(0) => random_test_menu(disk_size),
            Ok(1) => manual_test(disk_size),
            Ok(2) => disk_size = input_with_default("Disk size (minimum 1)", disk_size).min(1),
            Ok(3) => break,
            Ok(_) | Err(_) => (),
        }
    }
}

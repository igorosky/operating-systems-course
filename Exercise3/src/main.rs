mod virtual_memory_simulator;
mod fifo_virtual_memory_simulator;
mod opt_virtual_memory_simulator;
mod lru_virtual_memory_simulator;
mod rand_virtual_memory_simulator;
mod approximated_lru_virtual_memory_simulator;
mod simulator;

extern crate rand;
extern crate dialoguer;
extern crate num_format;

use std::str::FromStr;

use num_format::{ToFormattedString, Locale};
use rand::Rng;

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
        // println!("Accesses count: {}", result.get_requests_count().to_formatted_string(&Locale::fr));
        // println!("Memory size: {}", result.get_memory_size().to_formatted_string(&Locale::fr));
        println!("Page faults count: {}", result.get_page_faults_count().to_formatted_string(&Locale::fr));
        println!("============================")
    }
}

fn random_test_menu(memory_size: usize) {
    let minimum_number_of_accesses_in_test: usize = input_with_default("Minimum number of accesses in test", 10000);
    let maximum_number_of_accesses_in_test: usize = input_with_default("Maximum number of accesses in test", 1000000).max(minimum_number_of_accesses_in_test);
    let minimum_address: usize = input_with_default("Minimum pageId", 0);
    let maximum_address: usize = input_with_default("Maximum pageId", 1000).max(minimum_address);
    let locality_chance = input_with_default("Locality chance (1/1000)", 10);
    let locality_minimal_length = input_with_default("Locality minimal length", 10);
    let locality_maximal_length = input_with_default("Locality minimal length", 30).max(locality_minimal_length);
    let locality_minimal_page_count = input_with_default("Locality minimal page count", 2);
    let locality_maximal_page_count = input_with_default("Locality maximal page count", 5).max(locality_minimal_page_count);

    print_results(simulator::simulate_every(simulator::MemoryAccessRequests::from({
        let mut rng = rand::thread_rng();
        let accesses_count = rng.gen_range(minimum_number_of_accesses_in_test..=maximum_number_of_accesses_in_test);
        let mut accesses_list = Vec::with_capacity(accesses_count);
        let mut locality_length = 0;
        let mut locality_pages = Vec::new();
        let mut localities_count = 0;
        let mut total_locality_size = 0;
        while accesses_list.len() < accesses_count {
            if locality_length > 0 {
                total_locality_size += 1;
                locality_length -= 1;
                accesses_list.push(locality_pages[rng.gen_range(0..locality_pages.len())]);
                continue;
            }
            if rng.gen_range(0..1000) < locality_chance {
                localities_count += 1;
                locality_length = rng.gen_range(locality_minimal_length..=locality_maximal_length);
                let page_count = rng.gen_range(locality_minimal_page_count..=locality_maximal_page_count);
                locality_pages.clear();
                locality_pages.reserve(page_count);
                let mut pages_in_locality = std::collections::HashSet::new();
                while locality_pages.len() < page_count {
                    let value = rng.gen_range(minimum_address..=maximum_address);
                    if pages_in_locality.insert(value) {
                        locality_pages.push(value);
                    }
                }
                continue;
            }
            accesses_list.push(rng.gen_range(minimum_address..=maximum_address));
        }
        println!("Memory size: {}", memory_size.to_formatted_string(&Locale::fr));
        println!("Localities count: {}", localities_count.to_formatted_string(&Locale::fr));
        println!("Accesses count: {}", accesses_count.to_formatted_string(&Locale::fr));
        println!("Total locality size: {}", total_locality_size.to_formatted_string(&Locale::fr));
        accesses_list
    }), memory_size));
}

fn manual_test(memory_size: usize) {
    let mut accesses = Vec::with_capacity(input("Page accesses count"));
    for i in 1..=accesses.capacity() {
        println!("Access {}.", i);
        accesses.push(input::<usize, &str>("Page id"));
    }
    print_results(simulator::simulate_every(simulator::MemoryAccessRequests::from(accesses), memory_size));
}

fn main() {
    let mut memory_size = 10;
    loop {
        match dialoguer::Select::new()
            .items(&["Random tests", "Manual test", "Memory size", "Exit"])
            .with_prompt("Select option")
            .interact() {
            Ok(0) => random_test_menu(memory_size),
            Ok(1) => manual_test(memory_size),
            Ok(2) => memory_size = input_with_default("Memory size", memory_size),
            Ok(3) => break,
            Ok(_) | Err(_) => (),
        }
    }
}

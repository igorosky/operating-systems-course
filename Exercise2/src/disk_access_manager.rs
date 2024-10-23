use crate::task::Task;

pub trait DiskAccessManager {
    fn add_task(&mut self, position: usize, realtime: Option<usize>);
    fn simulate_n_ticks(&mut self, n: usize);
    fn finalize(self) -> Vec<Task>;
}

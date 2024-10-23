use rand::{distributions::uniform::SampleRange, Rng};
use getset::CopyGetters;

#[derive(Debug, Clone, CopyGetters)]
pub struct Process {
    #[getset(get_copy = "pub")]
    id: usize,
    #[getset(get_copy = "pub")]
    time_done: f64,
    #[getset(get_copy = "pub")]
    total_time: usize,
    #[getset(get_copy = "pub")]
    required_resources: usize,
    #[getset(get_copy = "pub")]
    waiting_time: usize,
    #[getset(get_copy = "pub")]
    send_count: usize,
    #[getset(get_copy = "pub")]
    working_time: usize,
}

impl PartialEq for Process {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Process { }

impl PartialOrd for Process {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.required_resources.partial_cmp(&other.required_resources) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.total_time.partial_cmp(&other.total_time) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.time_done.partial_cmp(&other.time_done) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.id.partial_cmp(&other.id) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.waiting_time.partial_cmp(&other.waiting_time) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.send_count.partial_cmp(&other.send_count)
    }
}

impl Ord for Process {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Process {
    #[inline]
    pub fn new_random<R: SampleRange<usize>>(id: usize, duration: R, required_resources: R) -> Self {
        let mut random = rand::thread_rng();

        Self {
            id,
            time_done: 0.0,
            total_time: random.gen_range(duration),
            required_resources: random.gen_range(required_resources),
            waiting_time: 0,
            send_count: 0,
            working_time: 0,
        }
    }

    #[inline]
    pub fn work(&mut self, efficiency: f64) {
        self.time_done += efficiency;
        self.working_time += 1;
    }

    #[inline]
    pub fn send(&mut self) {
        self.send_count += 1;
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.time_done as usize >= self.total_time
    }
    
    pub(crate) fn wait(&mut self) {
        self.waiting_time += 1;
    }
}

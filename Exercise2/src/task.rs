#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    UNFINISHED,
    STARVED,
    SUCCESSFUL(usize),
}

#[derive(Debug, Clone)]
pub struct Task {
    id: usize,
    position: usize,
    creation_time: usize,
    realtime: Option<usize>,
    done: State,
}

impl Task {
    #[inline]
    pub fn new(id: usize, position: usize, creation_time: usize, realtime: Option<usize>) -> Self {
        Self { id, position, creation_time, realtime, done: State::UNFINISHED }
    }

    #[inline]
    pub fn get_id(&self) -> usize {
        self.id
    }

    #[inline]
    pub fn get_creation_time(&self) -> usize {
        self.creation_time
    }
    
    #[inline]
    pub fn get_position(&self) -> usize {
        self.position
    }

    #[inline]
    pub fn is_realtime(&self) -> bool {
        self.realtime.is_some()
    }

    #[inline]
    pub fn get_realtime(&self) -> Option<usize> {
        self.realtime
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.done != State::UNFINISHED
    }

    #[inline]
    pub fn get_state(&self) -> State {
        self.done.clone()
    }

    pub fn finalize(&mut self, current_time: usize) -> Option<bool> {
        if self.is_done() {
            None
        }
        else if let Some(lifetime) = self.realtime {
            if self.creation_time + lifetime >= current_time {
                self.done = State::SUCCESSFUL(current_time);
                Some(true)
            }
            else {
                self.done = State::STARVED;
                Some(false)
            }
        }
        else {
            self.done = State::SUCCESSFUL(current_time);
            Some(true)
        }
    }

    pub fn set_starved(&mut self) {
        self.done = State::STARVED
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.position == other.position
    }
}

impl Eq for Task {
    
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.position.partial_cmp(&other.position) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.id.partial_cmp(&other.id) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        Some(std::cmp::Ordering::Equal)
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

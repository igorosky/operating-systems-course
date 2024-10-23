pub(super) trait ProcessSortableReminders {
    fn get_page_count(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn set_available_memory(&mut self, space: usize);
}

#[derive(Clone)]
pub(super) struct ReminderSorter {
    reminders: Vec<(usize, usize, usize)>,
    memory_size: usize,
}

impl ReminderSorter {
    #[inline]
    pub(super) fn new(memory_size: usize, len: usize) -> Self {
        Self {
            reminders: Vec::with_capacity(len),
            memory_size
        }
    }

    #[inline]
    pub(super) fn sort_reminders(arr: &mut [(usize, usize, usize)]) {
        arr.sort_unstable_by(|(reminder_a, eq_a, _), (reminder_b, eq_b, _)| {
            match reminder_b.cmp(reminder_a) {
                std::cmp::Ordering::Less => std::cmp::Ordering::Less,
                std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
                std::cmp::Ordering::Equal => eq_a.cmp(eq_b),
            }
        })
    }

    pub(super) fn set_frame_counts(&mut self, total_amount_of_pages: usize, processes: &mut [impl ProcessSortableReminders]) {
        if total_amount_of_pages == 0 {
            return;
        }
        let mut used_memory = 0;
        self.reminders.clear();
        for (i, process) in processes.iter_mut().enumerate() {
            let p = if process.is_empty() {
                0
            }
            else {
                process.get_page_count() * self.memory_size
            };
            let whole_frames = p / total_amount_of_pages;
            used_memory += whole_frames;
            self.reminders.push((
                p % total_amount_of_pages,
                whole_frames,
                i
            ));
        }
        Self::sort_reminders(&mut self.reminders);
        for (frames_count, i) in self.reminders
            .iter()
            .map(|(_, pages, i)| match used_memory < self.memory_size {
                true => {
                    used_memory += 1;
                    (*pages + 1, *i)
                }
                false => (*pages, *i),
            })
        {
            processes[i].set_available_memory(frames_count);
        }
        #[cfg(debug_assertions)]
        if used_memory > self.memory_size {
            panic!("Too much memory assigned");
        }
    }
}

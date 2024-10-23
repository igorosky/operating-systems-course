use std::{collections::{HashSet, LinkedList}, ops::RangeBounds, usize};

use rand::{distributions::uniform::SampleRange, Rng};


#[derive(Debug, Clone)]
pub struct Process {
    calls: LinkedList<Vec<usize>>,
    used_pages: Vec<usize>,
    pages_count: usize,
}

impl Process {
    pub(crate) fn new(
            total_len: impl SampleRange<usize> + RangeBounds<usize>,
            len: impl SampleRange<usize> + RangeBounds<usize> + Clone,
            pages: impl SampleRange<usize> + RangeBounds<usize> + Clone
        ) -> Self {
        let mut random = rand::thread_rng();
        let total_len = random.gen_range(total_len);
        let mut calls = LinkedList::new();
        let mut used_pages = HashSet::new();
        for _ in 0..total_len {
            let len = random.gen_range(len.clone());
            calls.push_back(
                Vec::from_iter(
                    (0..len)
                    .map(|_| {
                        let page = random.gen_range(pages.clone());
                        used_pages.insert(page);
                        page
                    })
                ));
        }
        let pages_begin = match pages.start_bound() {
            std::ops::Bound::Included(v) => *v,
            std::ops::Bound::Excluded(v) => *v + 1,
            std::ops::Bound::Unbounded => panic!("Process cannot have infinite amount of pages"),
        };
        let pages_end = match pages.end_bound() {
            std::ops::Bound::Included(v) => *v + 1,
            std::ops::Bound::Excluded(v) => *v,
            std::ops::Bound::Unbounded => panic!("Process cannot have infinite amount of pages"),
        };
        Self {
            calls,
            used_pages: used_pages.into_iter().collect(),
            pages_count: pages_end - pages_begin,
        }
    }

    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }

    #[inline]
    pub(super) fn next(&mut self) -> Option<Vec<usize>> {
        self.calls.pop_front()
    }

    #[inline]
    pub(super) fn get_page_count(&self) -> usize {
        self.pages_count
    }

    #[inline]
    pub(super) fn get_used_pages(&self) -> &Vec<usize> {
        &self.used_pages
    }
}

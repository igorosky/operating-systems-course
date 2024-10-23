pub trait VirtualMemorySimulator: Sized {
    fn new(memory_size: usize) -> Self;
    
    fn finalize(self) -> usize;

    fn get_page(&mut self, page_id: usize);
    
    #[inline]
    fn create_and_simulate<T: IntoIterator<Item = usize>>(memory_size: usize, memory_accesses: T) -> usize {
        let mut vm = Self::new(memory_size);
        memory_accesses.into_iter().for_each(|page_id| vm.get_page(page_id));
        vm.finalize()
    }
}

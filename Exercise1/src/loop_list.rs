// Rust's build in list does not provide every functionality for this code to be optimized - can't keep current iter and add element at the same time (because of borrow checker)

use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct ListNode<T> {
    next: Option<Rc<RefCell<ListNode<T>>>>,
    prev: Option<Rc<RefCell<ListNode<T>>>>,
    value: T,
}

impl<T> ListNode<T> {
    #[inline]
    fn new(val: T) -> Rc<RefCell<ListNode<T>>> {
        Rc::new(RefCell::new(ListNode { next: None, prev: None, value: val }))
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    fn finalize(self) -> T {
        self.value
    }
}

#[derive(Debug)]
pub struct LoopListIter<T> {
    node: Option<Rc<RefCell<ListNode<T>>>>,
    len: usize,
}

impl<T> LoopListIter<T> {
    #[inline]
    pub fn new() -> Self {
        Self { node: None, len: 0 }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.node.is_none()
    }

    pub fn add(&mut self, val: T) {
        match self.node.clone() {
            Some(node) => {
                let new_node = ListNode::new(val);
                let mut borrow = new_node.borrow_mut();
                borrow.next = node.borrow().next.clone();
                borrow.prev = Some(node.clone());
                match self.len {
                    1 => node.borrow_mut().prev = Some(new_node.clone()),
                    _ => node.borrow().next.as_ref().unwrap().borrow_mut().prev = Some(new_node.clone()),
                }
                drop(borrow);
                node.borrow_mut().next = Some(new_node);
            },
            None => {
                let node = ListNode::new(val);
                let mut borrow = node.borrow_mut();
                borrow.next = Some(node.clone());
                borrow.prev = Some(node.clone());
                drop(borrow);
                self.node = Some(node);
            },
        }
        self.len += 1;
    }

    pub fn erase(&mut self) -> Option<T> {
        let ans;
        match self.len {
            0 => return None,
            1 => {
                let mut node = self.node.as_ref().unwrap().borrow_mut();
                node.next = None;
                node.prev = None;
                drop(node);
                ans = self.node.take();
                self.len -= 1;
            },
            _ => {
                let mut node = self.node.as_ref().unwrap().borrow_mut();
                node.prev.as_ref().unwrap().borrow_mut().next = node.next.clone();
                node.next.as_ref().unwrap().borrow_mut().prev = node.prev.clone();
                let new_current = node.prev.take();
                drop(node);
                ans = self.node.take();
                self.node = new_current;
                self.len -= 1;
            },
        }
        Rc::try_unwrap(ans.unwrap()).ok().map(|v| v.into_inner().finalize())
    }

    #[inline]
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn next(&mut self) {
        if self.node.is_some() {
            let new_current_node = self.node.as_ref().unwrap().borrow().next.clone();
            self.node = new_current_node;
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn prev(&mut self) {
        if self.node.is_some() {
            let new_current_node = self.node.as_ref().unwrap().borrow().prev.clone();
            self.node = new_current_node;
        }
    }

    #[inline]
    pub fn get(&self) -> Option<Rc<RefCell<ListNode<T>>>> {
        self.node.clone()
    }
}

// and we are back to singly linked list(or queue i guess)
// but we are unsafe this time

use std::ptr;

pub struct List<T> {
    head: Link<T>,
    // this is a huge no no (unless we want to push just a single element here and never pop or even peek at it)
    // this is because what we are defining here is a mutable reference to a node that must be valid
    // as long as the list is active and because we have this we cannot push another element or pop
    // an element ever again
    // tail: Option<&'a mut Node<T>>,
    // so we move away from this to
    tail: *mut Node<T>,
}

// we are back to Box
type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: ptr::null_mut(),
        }
    }

    pub fn push(&mut self, elem: T) {
        let mut new_tail = Box::new(Node { elem, next: None });
        let raw_tail: *mut _ = &mut *new_tail;
        if !self.tail.is_null() {
            // SAFETY: not now
            unsafe {
                (*self.tail).next = Some(new_tail);
            }
        } else {
            self.head = Some(new_tail);
        }
        self.tail = raw_tail;
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            match old_head.next {
                None => {
                    // if the next node of head is none, the after popping that node
                    // the list is now empty
                    // but the tail pointed to the same popped element so we need to set the tail to null
                    self.tail = ptr::null_mut();
                }
                Some(new_head) => {
                    self.head = Some(new_head);
                }
            }
            old_head.elem
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_push_and_pop() {
        let mut list = List::<u8>::new();
        assert_eq!(list.pop(), None);
        list.push(23);
        list.push(42);
        assert_eq!(list.pop(), Some(23));
        assert_eq!(list.pop(), Some(42));
        assert_eq!(list.pop(), None);
    }
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }
}

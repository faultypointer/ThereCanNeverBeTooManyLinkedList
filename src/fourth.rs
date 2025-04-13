// INTERIOR MUTABILITY lets gooo!!
// so what is interior mutability?
// its a pattern provided by the compiler god allowing us to mutate data through immutable references
// RefCell<T> provideds one such pattern
// the rust mem. safety rules are checked at runtime (as opposed to compile time with most types)
//
// doubly linked list so two pointers in each node
// and two pointers in the list itself
//

use std::cell::RefCell;
use std::rc::Rc;

struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

// simple new node implementation
// so that its less clutter in the other places that needs to create new Node
impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem: elem,
            prev: None,
            next: None,
        }))
    }
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    // building the list
    // since we now have two pointers both in List and Node, its going to be more complicated than singly linked list
    // we have to consider both ends and what to put where
    //
    // pushing in the front
    // when pushing in the front if the list was empty
    // head and tail of list both needs to point to the new node
    // and if the list was not empty the head needs to point to the new node
    // but before that the old head must be pointed to by the new node's next
    // then the old head's prev should point to the new node
    // eg: [head] -> (prev: None, A, next) -> (prev: <-, B, next: None)
    // now if we want to add a new node with C at the front
    // first: (prev: None, C, next) -> (prev: None, A, next) ...
    // then: (prev: None, C, next) -> (prev: <-, A, next)...
    // finally: [head] -> (prev: None, C, next) -> (prev: <-, A, next)...
    // step 2 and 3 can be interchanged
    //
    // NOTE
    // at any point in time, any Node in the list will have the ref count of 2. that is two pointers pointing to it
    // when list is empty, there is no Node in the list
    // when we add a new node to empty list
    // 1. the head as well as tail points to it making the ref count 2
    // 2. when another list is added in front, the list head points to the new node and the old head node's prev filed also points to the new node
    //    making the new node's ref count 2 but the ref count of the old head node has decreased by 1 since the head no longer points to that node
    //    but but the new node's next points to the old head node thus making the count 2
    // 3. similar, when adding node to the back, list's tail points to the new node, new node's prev points to the old tail and old tail's next points to the new node
    //    making other the new node(tail) and old tails's ref count 0
    // 4. when adding in the middle of the list, the ref count of the new node's left and right node each decreases by 1 since the left's next and right's prev now points to the new node
    //    but the new node's prev points to the left node and new node's next points to the right node bringing their count to 2
    //
    // this will be important later when moving RefCell out of Rc
    //
    //
    // !! FAILED ATTEMPT !!
    // pub fn push_front(&mut self, elem: T) {
    //     let new_node = Node::new(elem);
    //     let prev = self.head.replace(Rc::clone(&new_node));
    //     match prev {
    //         None => {
    //             self.tail = Some(Rc::clone(&new_node));
    //         }
    //         Some(node) => {
    //             new_node.borrow_mut().next = Some(Rc::clone(&node));
    //             node.borrow_mut().prev = Some(Rc::clone(&new_node));
    //         }
    //     }
    // }

    // for pop front, its kinda similar
    // if head is empty return option
    // if its not we get the node from the head
    // make the head point to node's next
    // make the node's next (the new head)'s previous None
    // then make the node's next None
    // and i think thats it for pop front
    // no its not
    // what if there was just one element
    // then the pop leaves the list empty
    // but the list's tail is still pointing to that thing
    // so we need to check if the node to be popped's next is empty
    // if it is then we know after popping the list will be empty
    // so we whould also set the tail to None
    // i don't know to do drop the RC reference
    // is it just drop() i havent done that in the push_front either
    //
    // pub fn pop_front(&mut self) -> Option<T> {
    //     self.head.take().map(|node| {
    //         let node = node.into_inner();
    //         match node.next {
    //             None => self.tail = None,
    //             Some(new_head) => {
    //                 self.head = Some(Rc::clone(&new_head));
    //                 new_head.borrow_mut().prev = None;
    //             }
    //         }
    //         Some(node.elem)
    //     })?
    // }
    //
    // tried to implement without looking at the book failed. lets see what the book has to say
    //well the push front code was correct we a few adjustments
    //
    pub fn push_front(&mut self, elem: T) {
        let new_node = Node::new(elem);
        match self.head.take() {
            None => {
                // the order is important not for the logic of doubly linked list
                // but the second assignments moves the new_node so if it is done first
                // Rc::clone cannot be called on it afterwards
                self.tail = Some(Rc::clone(&new_node));
                self.head = Some(new_node);
            }
            Some(old_head) => {
                // same reason for the order here
                new_node.borrow_mut().next = Some(Rc::clone(&old_head));
                old_head.borrow_mut().prev = Some(Rc::clone(&new_node));
                self.head = Some(new_node);
            }
        }
    }

    // the pop front was also kinda the same but the problem i had (and the one explained in the book) was the
    // into_inner() which works on RefCell was not allowed because it was inside Rc
    // we need to get it out of Rc with try_unwrap
    // for which, the ref count must be one which is not the case initially but after dropping all the references to old head
    // we can take the the RefCell out of Rc
    pub fn pop_front(&mut self) -> Option<T> {
        // here we take the old head from self.head to old_head
        // so the ref count is still 2
        self.head.take().map(|old_head| {
            // checking if the list only had 1 element
            match old_head.borrow_mut().next.take() {
                // if so now the list should be empty thus the list's tail (which also pointed to the old head) must
                // now be None and reference be dropped
                None => {
                    // here we are dropping the reference to the old_head
                    // so in this branch the ref count of old_head is 1
                    self.tail.take();
                }
                // if the old_head's next was not None then we know the list has atleast 2 element before the pop
                // so no need to drop the reference of tail as it doesnt references the popped old_head
                Some(new_head) => {
                    // we do need to drop the reference of old_head from the new_head as it references
                    // the old_head with the prev field
                    // after dropping the ref count of old_head is also 1
                    new_head.borrow_mut().prev.take();
                    // don't forget make the list's head point to the new head
                    self.head = Some(new_head);
                }
            }
            // now we can take the RefCell out of Rc because the old_head ref count is only 1 in both branch of the previous match
            // try_unwrap returns a result so we need to unwrap it
            // unwrap need the underlying type to implement Debug and since Node doesnt, RefCell<Node> also doesnt
            // so we need ok to convert it to option whose unwrap doesn't need Debub
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem
        })
    }
}

// DROPPING: Rc and Cycle
// each adjacent two nodes in our list point to each other (A -> B and A <- B)
// so when the list goes out of scope, (unless it has only single node) it will not be dropped
// because there are still rec counts for all the nodes
// so we need to manually pop each element

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

#[cfg(test)]
mod tests {
    use super::List;
    #[test]
    fn front() {
        let mut list = List::new();
        assert_eq!(list.pop_front(), None);
        list.push_front(1);
        list.push_front(42);
        list.push_front(43);
        assert_eq!(list.pop_front(), Some(43));
        assert_eq!(list.pop_front(), Some(42));

        // Push some more just to make sure nothing's corrupted
        list.push_front(4);
        list.push_front(5);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }
}

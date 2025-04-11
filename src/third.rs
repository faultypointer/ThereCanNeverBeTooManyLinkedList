// we are implementing a "persistent immutable singly-linkedlist"
// we learn about Rc and Arc arey
// learning a little bit of common lisp came in handy I guess.
// I see what we are implementing
// we are implementing the cons but very simple one  i guess
// what we will be making will support this kinda stuff
// list1 = A -> B -> C -> D
// list2 = tail(list1) = B -> C -> D
// list3 = push(list2, X) = X -> B -> C -> D
//
// and will have following layout
// list1 -> A ---+
//               |
//               v
// list2 ------> B -> C -> D
//               ^
//               |
// list3 -> X ---+
// so how do we do it. Box can't help here. if the list2 is dropped how does compiler knows not to drop B and the list as
// list1 and list3 are still in play what we need is a way to count how many references to B exists and if it goes to 0 then we
// can legally drop B and friends. so we need some sort of a reference counter.
//
// According to rust docs "The type Rc<T> provides shared ownership of a value of type T, allocated in the heap. Invoking clone
// on Rc produces a new pointer to the same allocation in the heap. When the last Rc pointer to a given allocation is destroyed,
// the value stored in that allocation (often referred to as “inner value”) is also dropped."
// more about it here https://doc.rust-lang.org/std/rc/index.html
//
// so lets go ahead and build this thing
//

use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

pub struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }
    // since we cant mutate the list push and pop (especially) dont make sense
    // so prepend and tail is what well implement

    // what we want to do here is have a new list with head node's elem equal to elem
    // and its next to point to the self's head (by cloning it)
    pub fn prepend(&self, elem: T) -> List<T> {
        List {
            head: Some(Rc::new(Node {
                elem,
                next: self.head.clone(),
            })),
        }
    }

    // tail is the opposite
    // we want to return a list with the self's head removed and its next be the new list's head
    pub fn tail(&self) -> List<T> {
        List {
            head: self.head.as_ref().and_then(|node| node.next.clone()),
        }
    }

    // and what a tail without a head
    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }
}

// and now iterators or just iter for this one since no taking ownership and no mutable reference

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

// dropping the list
// in the last method we mutated the list in the drop the node in while loop
// we cant do that here because we only have shared reference
// as long as we are inside Rc
// so if we know that we are the only thing that references the node we can move the node
// out of the Rc
// Rc::try_unwrap() returns the inner value, if the Rc has only one strong pointer otherwire returns
// an Err
//
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(node) = head {
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // Make sure empty tail works
        let list = list.tail();
        assert_eq!(list.head(), None);
    }

    #[test]
    fn iter() {
        let list = List::new().prepend(1).prepend(2).prepend(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }
}

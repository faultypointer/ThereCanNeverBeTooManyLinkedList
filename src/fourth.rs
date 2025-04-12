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
}

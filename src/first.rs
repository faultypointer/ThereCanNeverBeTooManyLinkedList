// so what is a linked list (yes we are doing this)
// in the language of functional programming a linked list might be defined as follows
// A linked list is either Empty or an element followed by a list.
//
// ahh! a recursive defination and how can it be Empty or and Element (followed by a list)
// well thats were sum types comes into play.
// so what exactly is a sum type. according to wikipedia
// A sum type is a data structure used to hold a value that could take on several different, but fixed,
// types.
// basically an enum in rust.
//

// pub enum List {
//     Empty,
//     Elem(i32, List),
// }

// and there's an error. "recursive type `List` has infinite size"
// well the size of an enum is the size of its largest variant + the discriminant
// and the size of our largest element is what. thats 4 bytes for i32 and the size of list which is
// again 4 bytes + size of list + .... you see where im going with this.
// so the compiler is telling us to add indirection to break the cycle
// suggesting us to use Box
// so what the heck is a Box
// a Box<T> is basically a pointer that owns some memory in the heap of type T
//

// pub enum List {
//     Empty,
//     Elem(i32, Box<List>),
// }

// and this is also not good. infact its bad. why? lets see
// lets say we have a node with elements A, B, C then whats the memory layout of our list
// The first element with value A will be allocated on stack which would then point to element with
// value B on the heap like so
// [A, ptr] -> (B, ptr) -> (C, ptr) -> (Empty, *junk*)
// the junk is there so that it has to be ready to become an Elem at any point
// so our first element is in stack while other are in heap.
// the Empty takes some extra space just to say im not a valid element
// let us consider an alternative layout
// [ptr] -> (A, ptr) -> (B, ptr) -> (C, *null*)
//
// this has one obvious benefit that it doesnt need extra (unnecessary) space just to say im not
// actually a list element
//
// the other benefit comes from when we want to delete the first element
// in our implmentation we would have to copy the entire element of (B, ptr) from heap to stack replacing
// the element with value A
//
// in the alternative layout we can just copy the ptr from (A, ptr) which points to element B over
// the ptr in stack
// same thing when spliting the list
//
// there is a temporary bad idea with three enum variants introduced in the book which im not gonna
// write here. but the lesson is that the enums like
// enum Foo {
//     A
//     B(Contains a null pointer)
// }
// here the null pointer optimization kicks in to eliminate the space needed for the tag.
// the compiler can set whole enum variant A to 0's since B contains a null pointer.
// this is the reason why Option with types Vec, Box etc have no overhead
//
// so to have proper layyou and get that null pointer optimization we arrive at the following struct
//

// struct Node {
//     elem: i32,
//     next: List,
// }

// pub enum List {
//     Empty,
//     More(Box<Node>),
// }

// this gives us compiler warning saying that "type `Node` is more private than the item `List::More::0`"
// i guess the List being pub make the Node pub through the More variant but since Node is private
// hence the warnign

// finally (I think)

use std::mem;

pub struct List {
    head: Link,
}

enum Link {
    Empty,
    More(Box<Node>),
}

struct Node {
    elem: i32,
    next: Link,
}

impl List {
    pub fn new() -> Self {
        List { head: Link::Empty }
    }

    pub fn push(&mut self, elem: i32) {
        // well we dont directly store i32 in a list. it needs to be in a Node struct so
        let new_node = Box::new(Node {
            elem,
            // but what should next be. according to `Node` next is a Link. now we are appending to the end of a list
            // after which there are no elements. so it should be a Link::Empty, right?
            // apparently the book is going for the stack (the data structure not the memory) so pushing at the top
            // so not the end of the list buf beginning
            // so next should be whatever the previous list was
            // next: self.head says "cannot move out of `self.head which is behind a mutable reference`"
            // its saying that since `Link` doesn't implement copy move occurs.
            // do we want copy?
            // i don't think so
            // what do we actually want. we want the node self.head(which is a link)'s More to point to the new node and the new node's next to point to
            // whatever the head was pointing to before.
            // so how to do that. the book suggest using std::mem::replace
            // is this really the most common and appropriate way to do this.
            next: mem::replace(&mut self.head, Link::Empty),
        });

        self.head = Link::More(new_node);
    }

    // i assume pop also operates on the beginning of the list. so
    // remove element from head. assign next element to the head
    // then return the removed element.
    // what if the list is empty return None I guess
    // and don't have to do anything to head
    pub fn pop(&mut self) -> Option<i32> {
        // match &self.head {
        //     Link::Empty => None,
        //     Link::More(elem) => {
        //         mem::replace(&mut self.head, elem.next);
        //         Some(elem.elem)
        //     }
        // }
        // this does work burrow occurs in the match expression and in the replace thing it says cannot move out of shared self.head
        // removing the & also doesnot help lets see what the book has to say
        // the replace idea was correct the place to use was a little bit off
        match mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => None,
            Link::More(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }

        // the replace thing seems very handy. according to docs the std::mem::replace "Moves src into the referenced dest, returning the previous dest value."
        // in c to implement the pop we could have done this (removing the empty and options thing and also not counting for null)
        // Node node = self.head
        // self.head = node.next
        // return node.elem
        // why can't we do this in rust well we did just differently.
        // when matching in the first attempt match &self.head is burrowing immutably and the call to replace needs a mutable borrow
        // accoridng to rust borrowing rule we can either have multiple shared burrow or single mutable borrow
    }
}

// we are gonna have to implement drop ourselves. as we can't rely on compiler for tail recusion. when dropping the `Box<Node>` the tail recursion cannot happen since we have to deallocate the
// box pointer afterwards
impl Drop for List {
    fn drop(&mut self) {
        let mut current_link = mem::replace(&mut self.head, Link::Empty);
        while let Link::More(mut boxed_node) = current_link {
            current_link = mem::replace(&mut boxed_node.next, Link::Empty);
            // here boxed_node goes out of scope. and since we replaced its next with empty no unbounded recursion happens
        }
    }
}

#[cfg(test)] // tells the compiler to only compile whole test module when running cargo test
mod test {
    use crate::first::List;

    #[test]
    fn empty_pop_returns_none() {
        let mut list = List::new();
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn non_empty_pops() {
        let mut list = List::new();
        list.push(12); // [head] -> (12, *null*)
        list.push(20); // [head] -> (20, next) -> (12, *null*)
        list.push(70); // [head] -> (70, next) -> (20, next) -> (12, *null*)

        assert_eq!(list.pop(), Some(70));
        assert_eq!(list.pop(), Some(20));
        assert_eq!(list.pop(), Some(12));
        assert_eq!(list.pop(), None);
    }
}

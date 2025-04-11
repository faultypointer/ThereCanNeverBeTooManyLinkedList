// first list with so much more like
// - advance option use
// - generics
// - lifetimes
// - iterators

pub struct List<T> {
    head: Link<T>,
}

// how could I have not realized. this is just `Option<T>` so we can just type that
// enum Link {
//     Empty,
//     More(Box<Node>),
// }
type Link<T> = Option<Box<Node<T>>>;
// and now change every Empty to None and every More to Some
// now mem::replace(&mut option, None) can be replace with option.take()
// finally replace match option {Some => Some, None => None} with map method

// making node generic
// this is just adding a generic type T over the type T in elem instead of i32 and let the compiler
// or lsp tell you where you have to add more T
struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem,
            next: self.head.take(),
        });
        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }

    // finally a new feature (for a peeping tom)
    // if the head exists we return a reference to it
    // pub fn peek(&self) -> Option<&T> {
    //     self.head.map(|node| &node.elem)
    // }
    // the above code doesnt work cannot mode out of self.head which is behind shared ref.
    // according to docs apparently the map takes self
    // lets see what the book has to say
    // also since we are returning reference i guess lifetime is coming
    // okay so there is an as_ref method that demotes Option<T> to Option<&T> perfect for our usecase (also no lifetime i guess)
    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }

    // also the mutable version
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.elem)
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut current_link = self.head.take();
        while let Some(mut boxed_node) = current_link {
            current_link = boxed_node.next.take();
        }
    }
}

// implementing iterator
// iterators are defined on collection by implementing the iterator trait
// pub trait Iterator {
//     type Item;
//     fn next(&mut self) -> Option(Self::Item);
// }
// the Item is an associated type. whenever something implements Iterator it has to define what the Item type is
// its like generic but less awkward.
// we have to implement 3 different kinds of iterators
// - IntoIter - T
// - IterMut - &mut T
// - Iter - &T
//
// the book has some weird way to implement IntoIter. i dont get it

// new type wrapper around list to implement IntoIter
pub struct IntoIter<T>(List<T>);

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

// now the Iter
// we can't do the same thing as IntoIter for this as we return reference so pop can't be used
// so we manually need to keep track of where the cursor is on the list items
// and finally this is where the lifetime should come i think
// and yes the lsp is already giving me the error about missing lifetime parameters
// since we are returnign a reference we do need to specify for how long that reference should be valid
//

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

// and finally IterMut, which according to book is wild
// as book suggests take every thing in iter to itermut
pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<T> List<T> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn empty_pop_returns_none() {
        let mut list: List<i32> = List::new();
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn non_empty_pops() {
        let mut list = List::new();
        list.push(12);
        list.push(20);
        list.push(70);

        assert_eq!(list.pop(), Some(70));
        assert_eq!(list.pop(), Some(20));
        assert_eq!(list.pop(), Some(12));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);
        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));
    }

    #[test]
    fn peek_mut() {
        let mut list = List::new();

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek_mut(), Some(&mut 3));
        list.peek_mut().map(|value| *value = 42);

        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
    }
}

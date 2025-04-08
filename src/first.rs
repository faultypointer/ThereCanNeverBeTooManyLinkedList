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

pub enum List {
    Empty,
    Elem(i32, Box<List>),
}

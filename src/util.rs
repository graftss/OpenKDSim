

pub enum ListElt<T> {
  Cons(T, Box<ListElt<T>>),
  Nil,
}

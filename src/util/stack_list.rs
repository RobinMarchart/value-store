use std::ops::Deref;

pub struct Cons<'a, T> {
    prev: &'a StackList<'a, T>,
    val: T,
    depth: usize,
}

pub enum StackList<'a, T> {
    Nil,
    Cons(Cons<'a, T>),
}

impl<'a, T> StackList<'a, T> {
    pub fn push(& self, val: T) -> StackList<'_, T> {
        match self {
            StackList::Nil => StackList::Cons(Cons {
                prev: self,
                val,
                depth: 1,
            }),
            StackList::Cons(cons) => StackList::Cons(Cons {
                prev: self,
                val,
                depth: cons.depth + 1,
            }),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            StackList::Nil => 0,
            StackList::Cons(cons) => cons.depth,
        }
    }
    pub fn is_empty(&self)->bool{
        match self{
            StackList::Nil => true,
            StackList::Cons(_) => false,
        }
    }

    pub fn to_vec_mapped<F,R>(&self,mut f:F) -> Vec<R> where F: FnMut(&T)->R {
        let len = self.len();
        let mut vec = Vec::with_capacity(len);
        let slice = vec.spare_capacity_mut();
        let mut this = self;
        while let StackList::Cons(cons) = this {
            slice[cons.depth - 1].write(f(&cons.val));
            this = cons.prev
        }
        unsafe { vec.set_len(len) }
        vec
    }

}
impl<'a, T: Clone> StackList<'a, T> {
    pub fn to_vec(&self) -> Vec<T> {
        let len = self.len();
        let mut vec = Vec::with_capacity(len);
        let slice = vec.spare_capacity_mut();
        let mut this = self;
        while let StackList::Cons(cons) = this {
            slice[cons.depth - 1].write(cons.val.clone());
            this = cons.prev
        }
        unsafe { vec.set_len(len) }
        vec
    }
}
impl<'a, T:Deref<Target=U>,U:Clone> StackList<'a, T> {
    pub fn to_vec_deref(&self) -> Vec<U> {
        let len = self.len();
        let mut vec = Vec::with_capacity(len);
        let slice = vec.spare_capacity_mut();
        let mut this = self;
        while let StackList::Cons(cons) = this {
            slice[cons.depth - 1].write(cons.val.clone());
            this = cons.prev
        }
        unsafe { vec.set_len(len) }
        vec
    }
}

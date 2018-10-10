
use std::rc::Rc;
use std::cell::RefCell;
use recursive_types::List::Cons;
use recursive_types::List::Nil;

enum List<T> {
    Cons(T, Rc<List<T>>),
    Nil,
}

fn size<T>(lst: List<T>) -> usize {
    let mut res = 0;
    let mut aux = &lst;
    let mut is_done = false;
    while !is_done {
        match aux {
            Cons(_,l) => { aux = &l; }
            Nil => {is_done = true;}
        }
        if !is_done {
            res += 1;
        }
    }
    res
}

fn m() {
    let l = Cons(5, Rc::new(Cons(6, Rc::new(Nil))));
    size(l);
}

// this is not an issue; try again
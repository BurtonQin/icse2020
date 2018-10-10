fn call_using_ptr() {
    let assign_two = |mut x: i32| {
        let ptr: *mut i32 = &mut x as *mut i32;
        unsafe {
            *ptr = 2;
        }
    };
    let x = 5;
    assign_two(x);
}


fn m_with_ptr<F>( f: F ) -> i32
    where F: Fn(i32) -> i32
{
    f(0)
}


fn with_closure() {
    m_with_ptr(|x| {
        6
    });
}

fn id(x:i32) -> i32 {
    x
}

fn with_fn() {
    m_with_ptr(id);
}
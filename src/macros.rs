macro_rules! println_stderr {
    ($($arg:tt)*) => {
        {
            use std::io::Write;
            match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
                Ok(_) => {},
                Err(x) => panic!("Unable to write to stderr (file handle closed?): {}", x),
            }
        }
    }
}

macro_rules! init_array {
    ($ty:ty, $len:expr, $val:expr) => {
        {
            let mut array: [$ty; $len] = unsafe { ::std::mem::uninitialized() };
            for i in array.iter_mut() {
                unsafe { ::std::ptr::write(i, $val); }
            }
            array
        }
    }
}

macro_rules! init_array_fn {
    ($ty:ty, $len:expr, $val:expr) => {
        {
            let mut array: [$ty; $len] = unsafe { ::std::mem::uninitialized() };
            for (i, e) in array.iter_mut().enumerate() {
                unsafe { ::std::ptr::write(e, $val(i)); }
            }
            array
        }
    }
}

macro_rules! try {
    ($do:expr) => {
        {
            match $do {
                Ok(ret) => ret,
                Err(err) => {
                    println!("Error: {}", err);
                    process::exit(1);
                }
            }
        }
    }
}
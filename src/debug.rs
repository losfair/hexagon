pub static mut GLOBAL_DEBUG: bool = false;

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (if unsafe { ::debug::GLOBAL_DEBUG } { eprintln!( $($arg)* ) })
}

pub unsafe fn enable() {
    GLOBAL_DEBUG = true;
}

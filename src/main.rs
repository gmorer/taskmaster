use libc::write;
use libc::c_void;

fn main() {
    println!("Hello, world!");

    let arg: &[u8] = b"hello";
    let arg = arg.as_ptr() as *const c_void;
    unsafe {
        write(1, arg, 4);
    }
}

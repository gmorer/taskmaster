use libc::c_char;
use libc::c_void;
use libc::execve;
use libc::fork;
use libc::pid_t;
use libc::size_t;
use libc::strlen;
use libc::write;
use libc::INT_MAX;
use std::ffi::CString;
use std::ptr;

fn exec_child(path: &str, argv: Vec<String>) -> pid_t {
    let converted_path = CString::new(path).unwrap();
    let v:Vec<*const c_char> = argv.into_iter().map(|string| CString::new(string).unwrap().as_ptr()).collect();
    v.append(ptr::null() as *const char);
    unsafe {
        execve(converted_path.as_ptr(), v, ptr::null());
    }
    return 0;
}

fn main() {
    println!("Hello, world!");

    let arg = CString::new("hello").unwrap();
    let arg = arg.as_ptr() as *const c_void;
    let path = "/bin/ls";
    let args: Vec<String> = vec![String::from("-la"), String::from("-w")];
    unsafe {
        let len: size_t = strlen(arg as *const c_char) + 1;
        println!("len = {}", len);
        write(1, arg, 6);
        let child_pid: pid_t = match fork() {
            0 => exec_child(&path, args),
            e @ 1..=INT_MAX => e,
            e => e,
        };
        println!("\n{}", child_pid);
    }
}

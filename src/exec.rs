//use libc::pid_t;

//fn exec_child(path: &str, argv: Vec<String>) -> pid_t {
//    let converted_path = CString::new(path).unwrap();
//    let v:Vec<*const c_char> = argv.into_iter().map(|string| CString::new(string).unwrap().as_ptr()).collect();
//    v.append(ptr::null() as *const char);
//    unsafe {
//        execve(converted_path.as_ptr(), v, ptr::null());
//    }
//    return 0;
//}
//

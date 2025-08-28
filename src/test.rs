

mod tests {
    use std::ffi::{c_char, CString, c_int};
    use crate::get_episode::get_episode;

    // #[test]
    // fn test_1() {
    //     unsafe {
    //         let server_id = CString::new("1329879").expect("CString::new failed");
    //         let get_episode_ptr = get_episode(std::ptr::null(), server_id.as_ptr());
    //         let result = CString::from_raw(get_episode_ptr as *mut c_char).into_string().unwrap();
    //         println!("{}", &result);
    //     }
    // }

    use crate::search::search;

    #[test]
    fn test_2() {
        unsafe {
            let search_string = CString::new("ubel").expect("CString::new failed");
            let page_number = c_int::from(1);
            let get_episode_ptr = search(search_string.as_ptr(), &page_number as *const c_int);
            let result = CString::from_raw(get_episode_ptr as *mut c_char).into_string().unwrap();
            println!("{}", &result);
        }
    }
}
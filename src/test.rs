

mod tests {
    use std::ffi::{c_char, CString};
    use crate::get_episode;

    #[test]
    fn test_1() {
        unsafe {
            let episode_id = CString::new("1329879").expect("CString::new failed");
            let get_watch_ptr = get_episode::new(episode_id.as_ptr());
            let result = CString::from_raw(get_watch_ptr as *mut c_char).into_string().unwrap();
            println!("{}", &result);
        }
    }
}


mod tests {
    use std::ffi::{c_char, CString};
    use serde_json::{json, to_string};
    

    // use crate::search::search;

    // #[test]
    // fn test_2() {
    //     unsafe {
    //         let args = CString::new(to_string(&json!({
    //             "search": String::from("ubel"),
    //             "page": 1
    //         })).unwrap()).expect("CString::new failed");
            
    //         let get_episode_ptr = search(args.as_ptr());
    //         let result = CString::from_raw(get_episode_ptr as *mut c_char).into_string().unwrap();
    //         println!("{}", &result);
    //     }
    // }

    use crate::get_episode_list::get_episode_list;
    #[test]
    fn test_3() {
        unsafe {
            let args = CString::new(to_string(&json!({
                "id": "100".to_string(),
            })).unwrap()).expect("CString::new failed");
            let get_episode_ptr = get_episode_list(args.as_ptr());
            let result = CString::from_raw(get_episode_ptr as *mut c_char).into_string().unwrap();
            println!("{}", &result);
        }
    }


    // use crate::get_episode_server::get_episode_server;

    // #[test]
    // fn test_4() {
    //     unsafe {
    //         let args = CString::new(to_string(&json!({
    //             "id": "141568".to_string(),
    //         })).unwrap()).expect("CString::new failed");
    //         let get_episode_ptr = get_episode_server(args.as_ptr());
    //         let result = CString::from_raw(get_episode_ptr as *mut c_char).into_string().unwrap();
    //         println!("{}", &result);
    //     }
    // }

    // use crate::get_server::get_server;

    // #[test]
    // fn test_get_server() {
    //     unsafe {
    //         let args = CString::new(to_string(&json!({
    //             "id": String::from("672865"),
    //         })).unwrap()).expect("CString::new failed");
            
    //         let get_server_ptr = get_server(args.as_ptr());
    //         let result = CString::from_raw(get_server_ptr as *mut c_char).into_string().unwrap();
    //         println!("{}", &result);
    //     }
    // }
}
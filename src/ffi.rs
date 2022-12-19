use std::ffi::{self, c_int, c_void};

use crate::*;

#[no_mangle]
pub extern "C" fn parse_message(message_out: *mut *mut Message, message: *const u8) -> c_int {
    let msg = unsafe { Message::parse(std::slice::from_raw_parts(message, 50)).unwrap() };
    let msg_box = Box::new(msg);
    unsafe {
        *message_out = Box::into_raw(msg_box);
    }
    0
}

#[no_mangle]
pub extern "C" fn free_message(message: *mut *mut Message) -> c_int {
    let msg = unsafe { Box::from_raw(*message) };
    drop(*msg);
    unsafe {
        *message = std::ptr::null::<Message>() as *mut Message;
    }
    0
}

#[no_mangle]
pub extern "C" fn get_header(
    message: *mut *mut Message,
    header_name: *const u8,
    header_len: c_int,
    header_out: *const u8,
    header_out_len: c_int,
) -> c_int {
  let msg = unsafe { Box::from_raw(*message) };
  let _value = msg.header("TODO").unwrap();
  0
}

mod bindings;
use bindings::*;
use core::ffi::c_char;
use std::os::raw::c_void;
use std::ptr;

#[no_mangle]
pub extern "C" fn Agent_OnAttach(vm: *mut JavaVM, options: *mut c_char, _reserved: *mut c_void) {
    unsafe {
        let mut jvmti: *mut jvmtiEnv = ptr::null_mut();

        // Correctly call GetEnv directly via vm pointer
        let get_env_fn = (**vm).GetEnv.unwrap();
        let res = get_env_fn(
            vm,
            (&mut jvmti) as *mut *mut jvmtiEnv as *mut *mut c_void,
            JVMTI_VERSION_1_2 as jint,
        );

        // Example usage: Get all threads
        let mut thread_count: jint = 0;
        let mut threads: *mut jthread = ptr::null_mut();

        let err = (**jvmti).GetAllThreads.unwrap()(jvmti, &mut thread_count, &mut threads);

        println!(
            "✅ Successfully attached to JVM. Thread count: {}",
            thread_count
        );
    }
}

// This will be called when you load statically via -agentpath
#[no_mangle]
pub extern "C" fn Agent_OnLoad(vm: *mut JavaVM, options: *mut c_char, reserved: *mut c_void) {
    Agent_OnAttach(vm, options, reserved);
}

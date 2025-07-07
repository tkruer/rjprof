use bindings::*;
use core::ffi::c_char;
use std::os::raw::c_void;
use std::ptr;

use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::Mutex;
use std::time::Duration;

thread_local! {
    static ENTRY_TIMES: RefCell<HashMap<jmethodID, u64>> = RefCell::new(HashMap::new());
}

/// Newtype wrapper for JVMTI method IDs, so we can safely share across threads.
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
struct MethodId(jmethodID);
// SAFETY: jmethodID is a raw pointer; it is safe to send and share between threads.
unsafe impl Send for MethodId {}
unsafe impl Sync for MethodId {}

/// Per-method call count and total time.
#[derive(Clone, Copy)]
struct MethodStats {
    count: u64,
    total_nanos: u64,
}

static METHOD_STATS: Lazy<Mutex<HashMap<MethodId, MethodStats>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

extern "C" fn method_entry_callback(
    jvmti_env: *mut jvmtiEnv,
    _jni_env: *mut JNIEnv,
    thread: jthread,
    method: jmethodID,
) {
    unsafe {
        let mut nano: jlong = 0;
        (**jvmti_env).GetTime.unwrap()(jvmti_env, &mut nano);
        ENTRY_TIMES.with(|m| {
            m.borrow_mut().insert(method, nano as u64);
        });
    }
}

extern "C" fn method_exit_callback(
    jvmti_env: *mut jvmtiEnv,
    _jni_env: *mut JNIEnv,
    thread: jthread,
    method: jmethodID,
    _was_popped_by_exception: jboolean,
    _return_value: jvalue,
) {
    unsafe {
        let mut nano_exit: jlong = 0;
        (**jvmti_env).GetTime.unwrap()(jvmti_env, &mut nano_exit);
        ENTRY_TIMES.with(|m| {
            if let Some(nano_enter) = m.borrow_mut().remove(&method) {
                let dur = Duration::from_nanos((nano_exit as u64).saturating_sub(nano_enter));
                let mut stats = METHOD_STATS.lock().unwrap();
                let entry = stats.entry(MethodId(method)).or_insert(MethodStats {
                    count: 0,
                    total_nanos: 0,
                });
                entry.count += 1;
                entry.total_nanos += dur.as_nanos() as u64;
            }
        });
    }
}

extern "C" fn vm_death_callback(jvmti_env: *mut jvmtiEnv, _jni_env: *mut JNIEnv) {
    let mut stats: Vec<(MethodId, MethodStats)> = {
        let guard = METHOD_STATS.lock().unwrap();
        guard.iter().map(|(&m, st)| (m, *st)).collect()
    };
    stats.sort_by_key(|&(_, st)| std::cmp::Reverse(st.total_nanos / st.count));
    let top_n = std::cmp::min(stats.len(), 20);
    println!("\n=== Top {} slowest methods (avg ns) ===", top_n);
    let mut rows = Vec::with_capacity(top_n);
    let mut max_method_len = "Method".len();
    let mut max_calls_len = "Calls".len();
    let mut max_avg_len = "Avg(ns)".len();
    let mut max_total_len = "Total(ns)".len();
    for (MethodId(method), st) in stats.into_iter().take(top_n) {
        let (name, sig) = unsafe {
            let mut name_ptr: *mut c_char = std::ptr::null_mut();
            let mut sig_ptr: *mut c_char = std::ptr::null_mut();
            let res = (**jvmti_env).GetMethodName.unwrap()(
                jvmti_env,
                method,
                &mut name_ptr,
                &mut sig_ptr,
                std::ptr::null_mut(),
            );
            if res == jvmtiError_JVMTI_ERROR_NONE {
                (
                    CStr::from_ptr(name_ptr).to_string_lossy().into_owned(),
                    CStr::from_ptr(sig_ptr).to_string_lossy().into_owned(),
                )
            } else {
                ("<unknown>".to_string(), String::new())
            }
        };
        let method_str = format!("{}{}", name, sig);
        let calls_str = format!("{}", st.count);
        let avg_val = st.total_nanos / st.count;
        let avg_str = format!("{}ns", avg_val);
        let total_str = format!("{}ns", st.total_nanos);
        max_method_len = max_method_len.max(method_str.len());
        max_calls_len = max_calls_len.max(calls_str.len());
        max_avg_len = max_avg_len.max(avg_str.len());
        max_total_len = max_total_len.max(total_str.len());
        rows.push((method_str, calls_str, avg_str, total_str));
    }
    println!(
        "{:<method_width$} {:>calls_width$} {:>avg_width$} {:>total_width$}",
        "Method",
        "Calls",
        "Avg(ns)",
        "Total(ns)",
        method_width = max_method_len,
        calls_width = max_calls_len,
        avg_width = max_avg_len,
        total_width = max_total_len
    );
    println!(
        "{:-<method_width$} {:-<calls_width$} {:-<avg_width$} {:-<total_width$}",
        "",
        "",
        "",
        "",
        method_width = max_method_len,
        calls_width = max_calls_len,
        avg_width = max_avg_len,
        total_width = max_total_len
    );
    for (m, c, a, t) in rows {
        println!(
            "{:<method_width$} {:>calls_width$} {:>avg_width$} {:>total_width$}",
            m,
            c,
            a,
            t,
            method_width = max_method_len,
            calls_width = max_calls_len,
            avg_width = max_avg_len,
            total_width = max_total_len
        );
    }
}

extern "C" fn vm_init_callback(jvmti_env: *mut jvmtiEnv, _jni_env: *mut JNIEnv, _thread: jthread) {
    unsafe {
        let mut thread_count: jint = 0;
        let mut threads: *mut jthread = ptr::null_mut();
        let err = (**jvmti_env).GetAllThreads.unwrap()(jvmti_env, &mut thread_count, &mut threads);

        println!("✅ [VM_INIT] JVM thread count: {}", thread_count);
    }
}

#[no_mangle]
pub extern "C" fn Agent_OnAttach(vm: *mut JavaVM, _options: *mut c_char, _reserved: *mut c_void) {
    unsafe {
        let mut jvmti: *mut jvmtiEnv = ptr::null_mut();

        // Correctly call GetEnv directly via vm pointer
        let get_env_fn = (**vm).GetEnv.unwrap();
        let res = get_env_fn(
            vm,
            (&mut jvmti) as *mut *mut jvmtiEnv as *mut *mut c_void,
            JVMTI_VERSION_1_2 as jint,
        );
        let mut caps = std::mem::zeroed::<jvmtiCapabilities>();
        caps.set_can_generate_method_entry_events(1);
        caps.set_can_generate_method_exit_events(1);
        let err = (**jvmti).AddCapabilities.unwrap()(jvmti, &caps);
        if err != jvmtiError_JVMTI_ERROR_NONE {
            eprintln!("Failed to add JVMTI capabilities: {}", err);
        }
        let callbacks = jvmtiEventCallbacks {
            VMInit: Some(vm_init_callback),
            VMDeath: Some(vm_death_callback),
            MethodEntry: Some(method_entry_callback),
            MethodExit: Some(method_exit_callback),
            ..std::mem::zeroed()
        };
        let err = (**jvmti).SetEventCallbacks.unwrap()(
            jvmti,
            &callbacks,
            std::mem::size_of::<jvmtiEventCallbacks>() as jint,
        );
        if err != jvmtiError_JVMTI_ERROR_NONE {
            eprintln!("Failed to set JVMTI event callbacks: {}", err);
        }
        let err = (**jvmti).SetEventNotificationMode.unwrap()(
            jvmti,
            jvmtiEventMode_JVMTI_ENABLE,
            jvmtiEvent_JVMTI_EVENT_VM_INIT,
            ptr::null_mut(),
        );
        if err != jvmtiError_JVMTI_ERROR_NONE {
            eprintln!("Failed to enable VM_INIT event notifications: {}", err);
        }
        (**jvmti).SetEventNotificationMode.unwrap()(
            jvmti,
            jvmtiEventMode_JVMTI_ENABLE,
            jvmtiEvent_JVMTI_EVENT_VM_DEATH,
            ptr::null_mut(),
        );
        (**jvmti).SetEventNotificationMode.unwrap()(
            jvmti,
            jvmtiEventMode_JVMTI_ENABLE,
            jvmtiEvent_JVMTI_EVENT_METHOD_ENTRY,
            ptr::null_mut(),
        );
        (**jvmti).SetEventNotificationMode.unwrap()(
            jvmti,
            jvmtiEventMode_JVMTI_ENABLE,
            jvmtiEvent_JVMTI_EVENT_METHOD_EXIT,
            ptr::null_mut(),
        );
        println!("🔗 Agent attached, waiting for VM_INIT...");
    }
}

// This will be called when you load statically via -agentpath
#[no_mangle]
pub extern "C" fn Agent_OnLoad(vm: *mut JavaVM, options: *mut c_char, reserved: *mut c_void) {
    Agent_OnAttach(vm, options, reserved);
}

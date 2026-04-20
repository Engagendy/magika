use magika::{Builder, FileType, Session};
use serde_json::json;
use std::ffi::{c_char, c_uchar, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::Mutex;

struct SessionHandle {
    session: Mutex<Session>,
}

#[repr(C)]
pub struct MagikaSessionHandle {
    _private: [u8; 0],
}

fn into_c_string(value: String) -> *mut c_char {
    let sanitized = value.replace('\0', "\\u0000");
    match CString::new(sanitized) {
        Ok(value) => value.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

fn panic_json(message: &str) -> *mut c_char {
    into_c_string(json!({ "ok": false, "error": message }).to_string())
}

fn error_json(message: impl ToString) -> *mut c_char {
    into_c_string(json!({ "ok": false, "error": message.to_string() }).to_string())
}

fn success_json(file_type: FileType) -> *mut c_char {
    let info = file_type.info();
    let kind = match &file_type {
        FileType::Directory => "directory",
        FileType::Symlink => "symlink",
        FileType::Inferred(_) => "inferred",
        FileType::Ruled(_) => "ruled",
    };

    let content_type = file_type.content_type().map(|content_type| {
        let info = content_type.info();
        json!({
            "label": info.label,
            "mimeType": info.mime_type,
            "group": info.group,
            "description": info.description,
            "extensions": info.extensions,
            "isText": info.is_text
        })
    });

    into_c_string(
        json!({
            "ok": true,
            "kind": kind,
            "score": file_type.score(),
            "info": {
                "label": info.label,
                "mimeType": info.mime_type,
                "group": info.group,
                "description": info.description,
                "extensions": info.extensions,
                "isText": info.is_text
            },
            "contentType": content_type
        })
        .to_string(),
    )
}

fn create_default_session() -> *mut MagikaSessionHandle {
    match Session::new() {
        Ok(session) => {
            let handle = Box::new(SessionHandle {
                session: Mutex::new(session),
            });
            Box::into_raw(handle).cast::<MagikaSessionHandle>()
        }
        Err(_) => ptr::null_mut(),
    }
}

fn create_session(
    inter_threads: usize,
    intra_threads: usize,
    parallel_execution: bool,
) -> *mut MagikaSessionHandle {
    match Builder::default()
        .with_inter_threads(inter_threads)
        .with_intra_threads(intra_threads)
        .with_parallel_execution(parallel_execution)
        .build()
    {
        Ok(session) => {
            let handle = Box::new(SessionHandle {
                session: Mutex::new(session),
            });
            Box::into_raw(handle).cast::<MagikaSessionHandle>()
        }
        Err(_) => ptr::null_mut(),
    }
}

unsafe fn identify_path_inner(
    handle: *mut MagikaSessionHandle,
    path: *const c_char,
) -> *mut c_char {
    if handle.is_null() {
        return error_json("session pointer was null");
    }
    if path.is_null() {
        return error_json("path pointer was null");
    }

    let path = match CStr::from_ptr(path).to_str() {
        Ok(path) => path,
        Err(_) => return error_json("path was not valid UTF-8"),
    };

    let handle = &*handle.cast::<SessionHandle>();
    let mut session = match handle.session.lock() {
        Ok(session) => session,
        Err(_) => return error_json("session mutex was poisoned"),
    };

    match session.identify_file_sync(path) {
        Ok(file_type) => success_json(file_type),
        Err(err) => error_json(err),
    }
}

unsafe fn identify_bytes_inner(
    handle: *mut MagikaSessionHandle,
    data: *const c_uchar,
    len: usize,
) -> *mut c_char {
    if handle.is_null() {
        return error_json("session pointer was null");
    }
    if data.is_null() && len != 0 {
        return error_json("data pointer was null");
    }

    let bytes = if len == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(data, len)
    };
    let handle = &*handle.cast::<SessionHandle>();
    let mut session = match handle.session.lock() {
        Ok(session) => session,
        Err(_) => return error_json("session mutex was poisoned"),
    };

    match session.identify_content_sync(bytes) {
        Ok(file_type) => success_json(file_type),
        Err(err) => error_json(err),
    }
}

#[no_mangle]
pub extern "C" fn magika_session_new() -> *mut MagikaSessionHandle {
    match catch_unwind(create_default_session) {
        Ok(handle) => handle,
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn magika_session_new_with_threads(
    inter_threads: usize,
    intra_threads: usize,
    parallel_execution: bool,
) -> *mut MagikaSessionHandle {
    match catch_unwind(|| create_session(inter_threads, intra_threads, parallel_execution)) {
        Ok(handle) => handle,
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn magika_session_free(handle: *mut MagikaSessionHandle) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !handle.is_null() {
            drop(Box::from_raw(handle.cast::<SessionHandle>()));
        }
    }));
}

#[no_mangle]
pub unsafe extern "C" fn magika_identify_path_json(
    handle: *mut MagikaSessionHandle,
    path: *const c_char,
) -> *mut c_char {
    match catch_unwind(AssertUnwindSafe(|| identify_path_inner(handle, path))) {
        Ok(result) => result,
        Err(_) => panic_json("panic while identifying path"),
    }
}

#[no_mangle]
pub unsafe extern "C" fn magika_identify_bytes_json(
    handle: *mut MagikaSessionHandle,
    data: *const c_uchar,
    len: usize,
) -> *mut c_char {
    match catch_unwind(AssertUnwindSafe(|| identify_bytes_inner(handle, data, len))) {
        Ok(result) => result,
        Err(_) => panic_json("panic while identifying bytes"),
    }
}

#[no_mangle]
pub unsafe extern "C" fn magika_string_free(value: *mut c_char) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !value.is_null() {
            drop(CString::from_raw(value));
        }
    }));
}

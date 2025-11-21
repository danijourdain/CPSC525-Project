use core::panic;
use std::{ffi::CStr, io::ErrorKind};



#[repr(C)]
#[derive(Clone)]
pub struct OrderServer {
    id: usize,
    ptr: *const ()
}

unsafe extern "C" {
    pub fn addsilly(
        i: std::ffi::c_int,
        c: std::ffi::c_int,
    ) -> std::ffi::c_int;

    fn open_server(
        id: std::ffi::c_int,
        master: *const ()
    ) -> *const ();

    fn close_server(
        ptr: *const ()
    ) -> core::ffi::c_int;


    fn open_record(
        ptr: *const ()
    ) -> std::ffi::c_int;
    fn log_last_order(
        ptr: *const ()
    );
    fn flush_order(
        ptr: *const ()
    ) -> core::ffi::c_int;
    pub fn open_master_server() -> *const ();
    pub fn close_master_server(ptr: *const ()) -> core::ffi::c_int;
    fn query_regions() -> core::ffi::c_int;
    fn get_region_name(id: core::ffi::c_int) -> *const core::ffi::c_char;
    // fn set_sender(
    //     ptr: *const (),
    //     id: core::ffi::c_int
    // ) -> core::ffi::c_int;
    // fn set_recipient(
    //     ptr: *const (),
    //     id: core::ffi::c_int
    // ) -> core::ffi::c_int;
    fn set_money(
        ptr: *const (),
        id: core::ffi::c_int
    ) -> core::ffi::c_int;
    fn try_lock(ptr: *const (), val: u32) -> core::ffi::c_int;
    fn release_lock(ptr: *const (), val: u32);
    fn fetch_current_user(ptr: *const ()) -> u32;
}

#[repr(transparent)]
pub struct MasterOrderBook {
    handle: *const ()
}

impl MasterOrderBook {
    pub fn new() -> Self {
        Self {
            handle: unsafe { open_master_server() }
        }
    }
    pub fn available_regions() -> usize {
        return unsafe { query_regions() }.try_into().expect("Returned a negative number of regions.")
    }
}

impl Drop for MasterOrderBook {
    fn drop(&mut self) {
        // Here we will ignore the errors since we are closing
        // it anyways.
        unsafe { close_master_server(self.handle) };
    }
}


impl OrderServer {
    pub fn open(id: i32, master: &MasterOrderBook) -> Self {
        let ptr = unsafe { open_server(id, master.handle) };
        if ptr.is_null() {
            panic!("specified an invalid region.");
        }
        Self {
            id: id as usize,
            ptr
        }
    }
    pub fn get_name(&self) -> &str {
        let ptr = unsafe { get_region_name(self.id as i32) };
        let cstr = unsafe { CStr::from_ptr(ptr) };
        cstr.to_str().expect("Failed to read region name")
    }
    pub fn open_record(&self) -> std::io::Result<()> {

        let status = unsafe { open_record(self.ptr) };
        if status == -1 {
            return Err(std::io::Error::new(ErrorKind::AlreadyExists, "There is already a record open."));
        } else if status == -2 {
            return Err(std::io::Error::new(ErrorKind::ResourceBusy, "The resource is busy."));
        }

        Ok(())
    }
    pub fn flush_record(&self) -> std::io::Result<()> {

        let status = unsafe { flush_order(self.ptr) };
        if status == -1 {
            return Err(std::io::Error::new(ErrorKind::AlreadyExists, "There is already a record open."));
        }

        Ok(())
    }
    pub fn try_lock(&self, claimant: u32) -> std::io::Result<()> {
        let claim = unsafe { try_lock(self.ptr, claimant) };
        if claim != 1 {
            return Err(std::io::Error::last_os_error());
        } else {
            return Ok(());
        }
    }
    pub fn set_money(&self, value: i32) -> std::io::Result<()> {

        let status = unsafe { set_money(self.ptr, value) };
        if status == -1 {
            return Err(std::io::Error::new(ErrorKind::AlreadyExists, "There is already a record open."));
        }

        Ok(())
    }
    pub fn fetch_current_user(&self) -> u32 {
        unsafe { fetch_current_user(self.ptr) }
    }
    pub fn log_last_order(&self) {
        unsafe { log_last_order(self.ptr) };
    }
    pub fn release_lock(&self, id: u32) {
        unsafe { release_lock(self.ptr, id); }
    }
}


impl Drop for OrderServer {
    fn drop(&mut self) {
        unsafe { close_server(self.ptr) };
    }
}

// We assume that these are thread safe, which
// introduces the vulnerability.
unsafe impl Send for OrderServer {}
unsafe impl Sync for OrderServer {}





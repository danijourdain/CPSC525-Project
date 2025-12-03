use core::{ffi, panic};
use std::{ffi::{CStr, CString}, io::ErrorKind, time::Duration};


/// The order struct, we lay this out using the C
/// ABI which allows us to get it directly from the 
/// C backend without too much fidgeting.
#[repr(C)]
#[derive(Debug)]
pub struct Order {
    pub recipient: i32,
    pub sender: i32,
    pub money: i32,
    pub region: i32,
    status: i32
}

// DEFINE THE BINDINGS
unsafe extern "C" {
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
    fn flush_order(
        ptr: *const ()
    ) -> core::ffi::c_int;
    pub fn open_master_server() -> *const ();
    pub fn close_master_server(ptr: *const ()) -> core::ffi::c_int;
    fn query_regions() -> core::ffi::c_int;
    fn get_region_name(id: core::ffi::c_int) -> *const core::ffi::c_char;
    fn set_money(
        ptr: *const (),
        id: core::ffi::c_int
    ) -> core::ffi::c_int;
    fn set_sender(
        ptr: *const (),
        id: core::ffi::c_int
    ) -> core::ffi::c_int;
    fn set_recipient(
        ptr: *const (),
        id: core::ffi::c_int
    ) -> core::ffi::c_int;
    fn get_balance(
        ptr: *const (),
        region: core::ffi::c_int
    ) -> core::ffi::c_int;
    fn try_lock(ptr: *const (), password: *const core::ffi::c_char) -> core::ffi::c_int;
    fn release_lock(ptr: *const ());
    fn get_database_length(ptr: *const ()) -> i32;
    fn get_database_entry_at(ptr: *const (), position: ffi::c_int) -> Order;
}





/// An order server object. This
/// manages records for a specific region.
pub struct OrderServer {
    /// The region ID.
    id: usize,
    /// The pointer to the underlying C object.
    ptr: *const (),
    /// The pointer to the Master orderbook's C object.
    master_ptr: *const ()
}


/// A master order book, this manages
/// the ledger and the database which
/// are all processed on a background thread.
pub struct MasterOrderBook {
    /// The pointer to the underlying C object, this
    /// is needed to call methods.
    handle: *const (),
    /// The list of actual order servers available that
    /// have been intialized.
    servers: Vec<OrderServer>
}

impl MasterOrderBook {
    /// Opens a new order book.
    pub fn new() -> Self {
        let ptr = unsafe { open_master_server() };
        if ptr.is_null() {
            panic!("failed to initialize the master order book!");
        }


        // We just spin until the book is ready. This is
        // some nice logic that allows us to wait until the
        // records are populated, however it is not necessary.
        while unsafe { get_database_length(ptr) } == 0 {
            std::thread::sleep(Duration::from_millis(1));
        }


        

        Self {
            handle: ptr,
            servers: vec![]
        }
    }

    /// Gets the top-N most recent orders.
    pub fn get_top_n_orders(&self, mut n: usize) -> Vec<Order> {
        let mut buffer = vec![];
        let end = unsafe { get_database_length(self.handle) } as usize;
        if end == 0 {
            return vec![]; // nothing in the list.
        }

        // Make sure to prevent against overfetch, thank
        // you to Rust for catching these bugs.
        if n > end {
            n = end;
        }

        while end - n < end {
            // Lookup the database entry.
            buffer.push(unsafe { get_database_entry_at(self.handle, (end - n) as i32) });
            n -= 1;
        }
        buffer
    }

    /// Opens a new order server given the region ID.
    pub fn open_order_server(&mut self, id: i32) {
        let order = OrderServer::open(id, &*self);
        self.servers.push(order);
    }
    /// Gets the order server for a specific region.
    pub fn get_region_server(&self, region: i32) -> Option<&OrderServer> {
        self.servers.get(region as usize)
    }
    /// Gets the amount of available regions.
    pub fn available_regions() -> usize {
        return unsafe { query_regions() }.try_into().expect("Returned a negative number of regions.")
    }
}

impl Drop for MasterOrderBook {
    fn drop(&mut self) {

        // Force the sub-books to be dropped first for proper RAII.
        self.servers.clear();

        // Here we will ignore the errors since we are closing
        // it anyways.
        unsafe { close_master_server(self.handle) };
    }
}



/// This is a wrapper for functions that return -1 if 
/// they have an error, but else are "void"
fn call_io_based_error_fn<F>(func: F) -> std::io::Result<()>
where 
    F: FnOnce() -> i32
{
    let claim =  func();
    if claim == -1 {
        return Err(std::io::Error::last_os_error());
    } else {
        return Ok(());
    }
}


impl OrderServer {
    /// Opens a new order server given the region ID
    /// and a reference to the master order book.
    fn open(id: i32, master: &MasterOrderBook) -> Self {
        let ptr = unsafe { open_server(id, master.handle) };
        if ptr.is_null() {
            panic!("specified an invalid region.");
        }
        Self {
            id: id as usize,
            ptr,
            master_ptr: master.handle
        }
    }
    /// Retrieves the name of the order server.
    pub fn get_name(&self) -> &str {
        let ptr = unsafe { get_region_name(self.id as i32) };
        let cstr = unsafe { CStr::from_ptr(ptr) };
        cstr.to_str().expect("Failed to read region name")
    }
    /// Opens a new record for working.
    pub fn open_record(&self) -> std::io::Result<()> {

        let status = unsafe { open_record(self.ptr) };
        if status == -1 {
            return Err(std::io::Error::new(ErrorKind::AlreadyExists, "There is already a record open."));
        } else if status == -2 {
            return Err(std::io::Error::new(ErrorKind::ResourceBusy, "The resource is busy."));
        }

        Ok(())
    }
    /// Gets the current balance for the order server.
    pub fn get_balance(&self) -> i32 {
        unsafe { get_balance(self.master_ptr, self.id as i32) }
    }
    /// Flushes the record to the background.
    /// This delegates entirely to the C code.
    fn flush_record(&self) -> std::io::Result<()> {

        let status = unsafe { flush_order(self.ptr) };
        if status == -1 {
            return Err(std::io::Error::new(ErrorKind::AlreadyExists, "There is already a record open."));
        }

        Ok(())
    }
    /// Tries to lock the backend to get access to the order book.
    pub fn try_lock(&self, password: &str) -> std::io::Result<AcquiredOrderServer<'_>> {

        let cstr = CString::new(password)?;
        
        let claim = unsafe { try_lock(self.ptr, cstr.as_c_str().as_ptr()) };
        if claim != 1 {
            return Err(std::io::Error::last_os_error());
        } else {
            return Ok(AcquiredOrderServer {
                server: self
            });
        }
    }
    /// Sets the money field of the current order.
    fn set_money(&self, value: i32) -> std::io::Result<()> {

        let status = unsafe { set_money(self.ptr, value) };
        if status == -1 {
            return Err(std::io::Error::new(ErrorKind::AlreadyExists, "There is already a record open."));
        }

        Ok(())
    }
    /// Sets the sender of the currently open order.
    fn set_sender(&self, value: i32) -> std::io::Result<()> {
        call_io_based_error_fn(|| unsafe { set_sender(self.ptr, value) })
    }
    /// Sets the recipient on the currently open order.
    fn set_recipient(&self, value: i32) -> std::io::Result<()> {
        call_io_based_error_fn(|| unsafe { set_recipient(self.ptr, value) })
    }
    /// Releases the held lock.
    fn release_lock(&self) {
        unsafe { release_lock(self.ptr); }
    }
}

/// This leverages Rust's type system
/// to release the lock properly.
pub struct AcquiredOrderServer<'a> {
    server: &'a OrderServer
}

impl<'a> AcquiredOrderServer<'a> {
    // Implement the standard suite of commands.
    pub fn get_balance(&self) -> i32 {
        self.server.get_balance()
    }
    pub fn set_sender(&self, sender: i32) -> std::io::Result<()> {
        self.server.set_sender(sender)
    }
    pub fn set_recipient(&self, recipient: i32) -> std::io::Result<()> {
        self.server.set_recipient(recipient)
    }
    pub fn set_money(&self, money: i32) -> std::io::Result<()> {
        self.server.set_money(money)
    }
    pub fn flush_record(&self) -> std::io::Result<()> {
        self.server.flush_record()
    }
    pub fn open_record(&self) -> std::io::Result<()> {
        self.server.open_record()
    }
}

impl<'a> Drop for AcquiredOrderServer<'a> {
    fn drop(&mut self) {
        // We want to automatically release the
        // lock when we drop this object.
        self.server.release_lock();
    }
}

impl Drop for OrderServer {
    fn drop(&mut self) {
        // We assume that the master order book has
        // already been shut down or else this may
        // cause a degree of UB.
        unsafe { close_server(self.ptr) };
    }
}

// We assume that these are thread safe, which
// introduces the vulnerability. This is unsound and
// is partially what allows the vulnerability to take place.
unsafe impl Send for MasterOrderBook {}
unsafe impl Sync for MasterOrderBook {}





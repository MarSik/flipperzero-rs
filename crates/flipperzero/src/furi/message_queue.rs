use core::ffi::c_void;
use core::time::Duration;

use flipperzero_sys::furi::kernel::duration_to_ticks;
use flipperzero_sys::furi::message_queue;

use crate::furi;

/// MessageQueue provides a safe wrapper around the furi message queue primitive.
pub struct MessageQueue<M: Sized> {
    hnd: *const message_queue::FuriMessageQueue,
    _marker: core::marker::PhantomData<M>,
}

impl<M: Sized> MessageQueue<M> {
    /// Constructs a message queue with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            hnd: unsafe { message_queue::alloc(capacity, core::mem::size_of::<M>()) },
            _marker: core::marker::PhantomData::<M>,
        }
    }

    // Attempts to add the message to the end of the queue, waiting up to timeout ticks.
    pub fn put(&self, msg: M, timeout: Duration) -> furi::Result<()> {
        let mut msg = core::mem::ManuallyDrop::new(msg);
        let timeout_ticks = duration_to_ticks(timeout);

        let status = unsafe {
            message_queue::put(self.hnd, &mut msg as *mut _ as *const c_void, timeout_ticks)
        };

        status.err_or(())
    }

    // Attempts to read a message from the front of the queue within timeout ticks.
    pub fn get(&self, timeout: Duration) -> furi::Result<M> {
        let timeout_ticks = duration_to_ticks(timeout);
        let mut out = core::mem::MaybeUninit::<M>::uninit();
        let status =
            unsafe { message_queue::get(self.hnd, out.as_mut_ptr() as *mut c_void, timeout_ticks) };

        if status.is_ok() {
            Ok(unsafe { out.assume_init() })
        } else {
            Err(status)
        }
    }

    /// Returns the capacity of the queue.
    pub fn capacity(&self) -> usize {
        unsafe { message_queue::capacity(self.hnd) as usize }
    }

    /// Returns the number of elements in the queue.
    pub fn len(&self) -> usize {
        unsafe { message_queue::count(self.hnd) as usize }
    }

    /// Returns the number of free slots in the queue.
    pub fn space(&self) -> usize {
        unsafe { message_queue::space(self.hnd) as usize }
    }
}

impl<M: Sized> Drop for MessageQueue<M> {
    fn drop(&mut self) {
        // Drain any elements from the message queue, so any
        // drop handlers on the message element get called.
        while self.len() > 0 {
            match self.get(Duration::MAX) {
                Ok(msg) => drop(msg),
                Err(_) => break, // we tried
            }
        }

        unsafe { message_queue::free(self.hnd) }
    }
}
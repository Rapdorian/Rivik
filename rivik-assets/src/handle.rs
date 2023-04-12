//! Thread safe cache handle

use std::{any::Any, cell::Cell, sync::Arc};

pub struct Handle {
    inner: *mut Arc<dyn Any + Send + Sync>,
    rc: *mut Cell<usize>,
}

impl Handle {
    pub fn new<T: Any + Send + Sync>(t: T) -> Self {
        let arc: Arc<dyn Any + Send + Sync> = Arc::new(t);
        Self {
            inner: Box::leak(Box::new(arc)),
            rc: Box::leak(Box::new(Cell::new(1))),
        }
    }

    pub fn send(&self) -> Arc<dyn Any + Send + Sync> {
        Arc::clone(unsafe { &*self.inner })
    }

    pub fn downcast_ref<T: Send + Sync + 'static>(self) -> &T {
        let arc = unsafe { &*self.inner };
        let arc: Arc<T> = Arc::downcast(arc).unwrap();
        Arc::as_ref(&arc)
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        {
            let rc = unsafe { &*self.rc };
            rc.set(rc.take() + 1);
        }
        Self {
            inner: self.inner,
            rc: self.rc,
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        {
            let rc = unsafe { &*self.rc };
            rc.set(rc.take() - 1);

            if rc.get() == 0 {
                // drop this copy of the arc
                unsafe { Box::from_raw(self.inner) };

                //TODO: Make this aware that is is being cached and remove itself from the cache if
                //the only remaining reference is in the cache
            }
        }
    }
}

use ::jni::{errors::Result, JNIEnv};

pub mod future;
pub mod task;

pub fn init(env: &JNIEnv) -> Result<()> {
    task::jni::init(env)?;
    future::jni::init(env)?;
    Ok(())
}

pub(crate) mod jni {
    use ::jni::NativeMethod;
    use std::ffi::c_void;

    pub fn native(name: &str, sig: &str, fn_ptr: *mut c_void) -> NativeMethod {
        NativeMethod {
            name: name.into(),
            sig: sig.into(),
            fn_ptr,
        }
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    use std::{
        sync::{Arc, Mutex},
        task::{RawWaker, RawWakerVTable, Waker},
    };

    pub type TestWakerData = Mutex<bool>;

    unsafe fn test_waker_new(data: &Arc<TestWakerData>) -> RawWaker {
        let data_ptr = Arc::as_ptr(data);
        Arc::increment_strong_count(data_ptr);
        RawWaker::new(data_ptr as *const (), &VTABLE)
    }

    unsafe fn test_waker_clone(ptr: *const ()) -> RawWaker {
        let data_ptr = ptr as *const TestWakerData;
        Arc::increment_strong_count(data_ptr);
        RawWaker::new(data_ptr as *const (), &VTABLE)
    }

    unsafe fn test_waker_wake(ptr: *const ()) {
        let data_ptr = ptr as *const TestWakerData;
        let data = &*data_ptr;
        let mut lock = data.lock().unwrap();
        *lock = true;
    }

    unsafe fn test_waker_wake_by_ref(ptr: *const ()) {
        test_waker_wake(ptr)
    }

    unsafe fn test_waker_drop(ptr: *const ()) {
        let data_ptr = ptr as *const TestWakerData;
        Arc::decrement_strong_count(data_ptr);
    }

    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        test_waker_clone,
        test_waker_wake,
        test_waker_wake_by_ref,
        test_waker_drop,
    );

    pub fn test_waker(data: &Arc<TestWakerData>) -> Waker {
        unsafe { Waker::from_raw(test_waker_new(data)) }
    }
}

#[cfg(test)]
mod test {
    use std::{sync::Arc, task::Waker};

    #[test]
    fn test_raw_waker_refcount() {
        let data = Arc::new(crate::test_utils::TestWakerData::new(false));
        assert_eq!(Arc::strong_count(&data), 1);

        let waker: Waker = crate::test_utils::test_waker(&data);
        assert_eq!(Arc::strong_count(&data), 2);
        assert_eq!(*data.lock().unwrap(), false);

        let waker2 = waker.clone();
        assert_eq!(Arc::strong_count(&data), 3);
        assert_eq!(*data.lock().unwrap(), false);

        std::mem::drop(waker2);
        assert_eq!(Arc::strong_count(&data), 2);
        assert_eq!(*data.lock().unwrap(), false);

        std::mem::drop(waker);
        assert_eq!(Arc::strong_count(&data), 1);
        assert_eq!(*data.lock().unwrap(), false);
    }

    #[test]
    pub fn test_raw_waker_wake() {
        let data = Arc::new(crate::test_utils::TestWakerData::new(false));

        let waker: Waker = crate::test_utils::test_waker(&data);
        assert_eq!(*data.lock().unwrap(), false);

        waker.wake();
        assert_eq!(*data.lock().unwrap(), true);
    }

    #[test]
    pub fn test_raw_waker_wake_by_ref() {
        let data = Arc::new(crate::test_utils::TestWakerData::new(false));

        let waker: Waker = crate::test_utils::test_waker(&data);
        assert_eq!(*data.lock().unwrap(), false);

        waker.wake_by_ref();
        assert_eq!(*data.lock().unwrap(), true);
    }
}

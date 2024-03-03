#[cfg(any(doc, target_arch = "wasm32"))]
pub mod not_send {
    use std::{
        cell::UnsafeCell,
        collections::VecDeque,
        fmt::Debug,
        ops::{Deref, DerefMut},
        sync::Arc,
        task::{Context, Poll, Waker},
    };

    use futures_util::{future::FusedFuture, Future};

    pub trait MaybeSend {}
    impl<T> MaybeSend for T {}
    pub trait MaybeSync {}
    impl<T> MaybeSync for T {}
    pub struct Mutex<T> {
        locked: UnsafeCell<bool>,
        content: UnsafeCell<T>,
        queue: UnsafeCell<VecDeque<Option<(usize, Waker)>>>,
        counter: UnsafeCell<usize>,
    }

    pub struct MutexGuard<'l, T> {
        inner: &'l Mutex<T>,
    }

    pub struct MappedMutexGuard<'l, T, M: ?Sized> {
        inner: &'l Mutex<T>,
        value: *mut M,
    }

    pub struct OwnedMutexGuard<T> {
        inner: Arc<Mutex<T>>,
    }

    #[derive(Debug)]
    pub struct MutexLockFuture<'l, T> {
        inner: &'l Mutex<T>,
        id: Option<usize>,
        fused: bool,
    }

    #[derive(Debug)]
    pub struct OwnedMutexLockFuture<T> {
        inner: Arc<Mutex<T>>,
        id: Option<usize>,
        fused: bool,
    }

    impl<T> Mutex<T> {
        pub fn new(t: T) -> Self {
            Mutex {
                locked: UnsafeCell::new(false),
                content: UnsafeCell::new(t),
                queue: UnsafeCell::new(VecDeque::new()),
                counter: UnsafeCell::new(0),
            }
        }

        pub fn into_inner(self) -> T {
            self.content.into_inner()
        }

        pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
            unsafe {
                if *self.locked.get() {
                    None
                } else {
                    Some(MutexGuard { inner: self })
                }
            }
        }

        pub fn try_lock_owned(self: &Arc<Self>) -> Option<OwnedMutexGuard<T>> {
            unsafe {
                if *self.locked.get() {
                    None
                } else {
                    Some(OwnedMutexGuard {
                        inner: self.clone(),
                    })
                }
            }
        }

        pub fn lock(&self) -> MutexLockFuture<'_, T> {
            MutexLockFuture {
                inner: self,
                id: None,
                fused: false,
            }
        }
        pub fn lock_owned(self: Arc<Self>) -> OwnedMutexLockFuture<T> {
            OwnedMutexLockFuture {
                inner: self,
                id: None,
                fused: false,
            }
        }

        pub fn get_mut(&mut self) -> &mut T {
            self.content.get_mut()
        }

        unsafe fn lock_inner(&self, id: &mut Option<usize>, cx: &mut Context<'_>) -> Poll<()> {
            let locked = &mut *self.locked.get();
            let queue = &mut *self.queue.get();
            match (*locked, *id) {
                (false, None) => {
                    *locked = true;
                    Poll::Ready(())
                }
                (false, Some(id)) => {
                    *locked = true;
                    remove_waker(queue, id);
                    Poll::Ready(())
                }
                (true, None) => {
                    let counter = &mut *self.counter.get();
                    let new_id = *counter;
                    *counter += 1;
                    queue.push_back(Some((new_id, cx.waker().to_owned())));
                    *id = Some(new_id);
                    Poll::Pending
                }
                (true, Some(_)) => Poll::Pending,
            }
        }
        unsafe fn drop_future(&self, id: usize) {
            let queue = &mut *self.queue.get();
            remove_waker(queue, id);
            if !*self.locked.get() {
                if let Some(Some((_, waker))) = queue.front().as_ref() {
                    waker.wake_by_ref();
                }
            }
        }
        unsafe fn unlock_innner(&self) {
            let queue = &mut *self.queue.get();
            if let Some(Some((_, waker))) = queue.front().as_ref() {
                waker.wake_by_ref();
            }
            *self.locked.get() = false;
        }
    }

    impl<T> Debug for Mutex<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Mutex")
                .field("locked", &unsafe { *self.locked.get() })
                .field("counter", &unsafe { *self.counter.get() })
                .finish()
        }
    }

    impl<'l, T> MutexGuard<'l, T> {
        pub fn map<U: ?Sized, F>(this: Self, f: F) -> MappedMutexGuard<'l, T, U>
        where
            F: FnOnce(&mut T) -> &mut U,
        {
            let mutex = this.inner;
            let val = f(unsafe { &mut *mutex.content.get() });
            std::mem::forget(this);
            MappedMutexGuard {
                inner: mutex,
                value: val,
            }
        }
    }
    impl<'l, T, M: ?Sized> MappedMutexGuard<'l, T, M> {
        pub fn map<U: ?Sized, F>(this: Self, f: F) -> MappedMutexGuard<'l, T, U>
        where
            F: FnOnce(&mut M) -> &mut U,
        {
            let mutex = this.inner;
            let val = f(unsafe { &mut *this.value });
            std::mem::forget(this);
            MappedMutexGuard {
                inner: mutex,
                value: val,
            }
        }
    }

    impl<'l, T: Debug> Debug for MutexGuard<'l, T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Debug::fmt(self.deref(), f)
        }
    }

    impl<T: Debug> Debug for OwnedMutexGuard<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Debug::fmt(self.deref(), f)
        }
    }

    impl<'l, T, M: Debug + ?Sized> Debug for MappedMutexGuard<'l, T, M> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Debug::fmt(self.deref(), f)
        }
    }

    impl<'l, T> Deref for MutexGuard<'l, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            unsafe { &*self.inner.content.get() }
        }
    }

    impl<T> Deref for OwnedMutexGuard<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            unsafe { &*self.inner.content.get() }
        }
    }

    impl<'l, T, M: ?Sized> Deref for MappedMutexGuard<'l, T, M> {
        type Target = M;

        fn deref(&self) -> &Self::Target {
            unsafe { &*self.value }
        }
    }

    impl<'l, T> DerefMut for MutexGuard<'l, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { &mut *self.inner.content.get() }
        }
    }

    impl<T> DerefMut for OwnedMutexGuard<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { &mut *self.inner.content.get() }
        }
    }

    impl<'l, T, M: ?Sized> DerefMut for MappedMutexGuard<'l, T, M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { &mut *self.value }
        }
    }

    impl<'l, T> Drop for MutexGuard<'l, T> {
        fn drop(&mut self) {
            unsafe { self.inner.unlock_innner() }
        }
    }

    impl<T> Drop for OwnedMutexGuard<T> {
        fn drop(&mut self) {
            unsafe { self.inner.unlock_innner() }
        }
    }

    impl<'l, T, M: ?Sized> Drop for MappedMutexGuard<'l, T, M> {
        fn drop(&mut self) {
            unsafe { self.inner.unlock_innner() }
        }
    }

    impl<'l, T> Drop for MutexLockFuture<'l, T> {
        fn drop(&mut self) {
            if let Some(id) = self.id {
                unsafe { self.inner.drop_future(id) }
            }
        }
    }

    impl<T> Drop for OwnedMutexLockFuture<T> {
        fn drop(&mut self) {
            if let Some(id) = self.id {
                unsafe { self.inner.drop_future(id) }
            }
        }
    }

    impl<'l, T> Future for MutexLockFuture<'l, T> {
        type Output = MutexGuard<'l, T>;

        fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let s = self.get_mut();
            assert!(!s.fused, "fused");
            if let Poll::Ready(()) = unsafe { s.inner.lock_inner(&mut s.id, cx) } {
                s.id = None;
                s.fused = true;
                Poll::Ready(MutexGuard { inner: s.inner })
            } else {
                Poll::Pending
            }
        }
    }
    impl<T> Future for OwnedMutexLockFuture<T> {
        type Output = OwnedMutexGuard<T>;

        fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let s = self.get_mut();
            assert!(!s.fused, "fused");
            if let Poll::Ready(()) = unsafe { s.inner.lock_inner(&mut s.id, cx) } {
                s.id = None;
                s.fused = true;
                Poll::Ready(OwnedMutexGuard {
                    inner: s.inner.clone(),
                })
            } else {
                Poll::Pending
            }
        }
    }

    impl<'l, T> FusedFuture for MutexLockFuture<'l, T> {
        fn is_terminated(&self) -> bool {
            self.fused
        }
    }

    impl<T> FusedFuture for OwnedMutexLockFuture<T> {
        fn is_terminated(&self) -> bool {
            self.fused
        }
    }

    /**
     *  removes waker with id. ensures that the first element is always Some (if it exists)
     *  */
    fn remove_waker(queue: &mut VecDeque<Option<(usize, Waker)>>, id: usize) {
        if let Some(Some((front_id, _))) = queue.front().as_ref() {
            if *front_id == id {
                queue.pop_front();
                while let Some(None) = queue.front().as_ref() {
                    queue.pop_front();
                }
            } else {
                let index = id.wrapping_sub(*front_id);
                let self_waker = queue.get_mut(index)
                                      .expect("wakers are only removed on successful lock or drop. after this function succeeds it should never be called again.")
                                      ;
                assert!(self_waker.is_some(),"wakers are only removed on successful lock or drop. after this function succeeds it should never be called again.");
                assert_eq!(
                    self_waker.as_ref().unwrap().0,
                    id,
                    "calculated the right index"
                );
                *self_waker = None;
            }
        } else {
            panic!("wakers are only removed on successful lock or drop. after this function succeeds it should never be called again.")
        }
    }
}
#[doc(no_inline)]
#[cfg(target_arch = "wasm32")]
pub use not_send::{
    MappedMutexGuard, MaybeSend, MaybeSync, Mutex, MutexGuard, MutexLockFuture, OwnedMutexGuard,
    OwnedMutexLockFuture,
};

#[cfg(any(not(target_arch = "wasm32"), doc))]
pub mod send {
    pub trait MaybeSend: Send {}
    impl<T: Send> MaybeSend for T {}
    pub trait MaybeSync: Sync {}
    impl<T: Sync> MaybeSync for T {}
}

#[doc(no_inline)]
#[cfg(not(target_arch = "wasm32"))]
pub use send::{MaybeSend, MaybeSync};

#[doc(no_inline)]
#[cfg(not(target_arch = "wasm32"))]
pub use futures_util::lock::{
    MappedMutexGuard, Mutex, MutexGuard, MutexLockFuture, OwnedMutexGuard, OwnedMutexLockFuture,
};

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use critical_section::Mutex;

pub struct GiveAwayCell<T>(UnsafeCell<MaybeUninit<T>>);

unsafe impl<T> Send for GiveAwayCell<T> {}
unsafe impl<T> Sync for GiveAwayCell<T> {}

impl Default for GiveAwayCell<MaybeUninit<()>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> GiveAwayCell<T> {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    /// # Safety
    ///
    /// You should write only once, before enabling interruptions
    pub unsafe fn write(&self, value: T) {
        *self.0.get() = MaybeUninit::new(value);
    }

    /// # Safety
    ///
    /// You should call assume_init or assume_init_mut in only one context (interruption),
    /// given it has been initialized before
    pub unsafe fn assume_init(&self) -> &T {
        let mu = &*self.0.get();
        mu.assume_init_ref()
    }

    /// # Safety
    ///
    /// You should call assume_init or assume_init_mut in only one context (interruption),
    /// given it has been initialized before
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn assume_init_mut(&self) -> &mut T {
        let mu = &mut *self.0.get();
        mu.assume_init_mut()
    }
}

pub struct SharedCell<T>(Mutex<UnsafeCell<MaybeUninit<T>>>);

impl Default for SharedCell<MaybeUninit<()>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SharedCell<T> {
    pub const fn new() -> Self {
        Self(Mutex::new(UnsafeCell::new(MaybeUninit::uninit())))
    }

    pub fn write(&self, value: T) {
        critical_section::with(|cs| {
            let cell = self.0.borrow(cs);
            unsafe { *cell.get() = MaybeUninit::new(value); }
        })

    }

    /// # Safety
    ///
    /// You should call lock only if it has been initialized before
    pub unsafe fn lock<F: FnOnce(&mut T) -> R, R>(&self, f: F) -> R {
        critical_section::with(|cs| {
            let cell = self.0.borrow(cs);
            let data = unsafe { (*cell.get()).assume_init_mut() };
            f(data)
        })
    }
}

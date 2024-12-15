#![no_std]

use core::cell::{RefCell, UnsafeCell};
use core::mem::MaybeUninit;
use critical_section::Mutex;

pub trait CellToken {
    type Inner;
}

pub trait Given: CellToken {}
pub trait Taken: Given {}

pub struct GiveAwayCell<CT: CellToken>(UnsafeCell<MaybeUninit<CT::Inner>>);

unsafe impl<CT: CellToken> Send for GiveAwayCell<CT> {}
unsafe impl<CT: CellToken> Sync for GiveAwayCell<CT> {}

impl<CT: CellToken> Default for GiveAwayCell<CT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<CT: CellToken> GiveAwayCell<CT> {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }
}

impl<CT: Given> GiveAwayCell<CT> {
    pub fn give(&self, value: CT::Inner) {
        unsafe {
            *self.0.get() = MaybeUninit::new(value);
        }
    }
}

impl<CT: Taken> GiveAwayCell<CT> {
    pub fn take(&self) -> &'static mut CT::Inner {
        let mu = unsafe { &mut *self.0.get() };
        unsafe { mu.assume_init_mut() }
    }
}

pub struct SharedCell<CT: CellToken>(Mutex<RefCell<MaybeUninit<CT::Inner>>>);

impl<CT: CellToken> Default for SharedCell<CT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<CT: CellToken> SharedCell<CT> {
    pub const fn new() -> Self {
        Self(Mutex::new(RefCell::new(MaybeUninit::uninit())))
    }
}

impl<CT: Given> SharedCell<CT> {
    pub fn give(&self, value: CT::Inner) {
        critical_section::with(|cs| {
            let cell = self.0.borrow(cs);
            cell.replace(MaybeUninit::new(value));
        })
    }

    pub fn with<F: FnOnce(&mut CT::Inner) -> R, R>(&self, f: F) -> R {
        critical_section::with(|cs| {
            let cell = self.0.borrow(cs);
            let mut mu = cell.borrow_mut();
            let data = unsafe { mu.assume_init_mut() };
            f(data)
        })
    }
}

pub use embed_init_macros::give_away_cell;
pub use embed_init_macros::shared_cell;

#[macro_export]
macro_rules! give {
    ($token:ident = $value:expr) => {{
        #[allow(non_local_definitions)]
        impl ::embed_init::Given for $token::Token {}
        $token::CELL.give($value)
    }};
}

#[macro_export]
macro_rules! take {
    ($token:ident) => {{
        #[allow(non_local_definitions)]
        impl ::embed_init::Taken for $token::Token {}
        $token::CELL.take()
    }};
}

#[macro_export]
macro_rules! borrow {
    ($token:ident, $f:expr) => {
        $token::CELL.with($f)
    };
}

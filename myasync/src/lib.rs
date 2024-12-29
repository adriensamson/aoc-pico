#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub struct Executor<Idle: Fn(), const N: usize> {
    tasks: [Task; N],
    idle: Idle,
}

impl<Idle: Fn(), const N: usize> Executor<Idle, N> {
    pub fn new(futures: [Pin<Box<dyn Future<Output=()>>>; N], idle: Idle) -> Self {
        Self {
            tasks: futures.map(Task::new),
            idle,
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            for task in &mut self.tasks {
                match task.state {
                    TaskState::Finished | TaskState::Waiting => {},
                    TaskState::NotStarted | TaskState::Awakened => {
                        task.state = TaskState::Waiting;
                        let waker = task.waker();
                        let mut ctx = Context::from_waker(&waker);
                        let fut = task.future.as_mut();
                        match fut.poll(&mut ctx) {
                            Poll::Ready(()) => {
                                task.state = TaskState::Finished;
                            },
                            Poll::Pending => {}
                        }
                    },
                }
            }
            if self.tasks.iter().all(|t| matches!(t.state, TaskState::Finished | TaskState::Waiting)) {
                (self.idle)();
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum TaskState {
    NotStarted,
    Waiting,
    Awakened,
    Finished,
}

struct Task {
    future: Pin<Box<dyn Future<Output=()>>>,
    state: TaskState,
}

impl Task {
    fn new(future: Pin<Box<dyn Future<Output=()>>>) -> Self {
        Self {
            future,
            state: TaskState::NotStarted,
        }
    }

    fn waker(&self) -> Waker {
        let data = (&raw const *self).cast();
        unsafe { Waker::from_raw(RawWaker::new(data, &RAW_WAKER_VTABLE)) }
    }
}

fn waker_clone(data: *const ()) -> RawWaker {
    RawWaker::new(data, &RAW_WAKER_VTABLE)
}
fn waker_wake(data: *const ()) {
    let task = data.cast::<Task>().cast_mut();
    let task = unsafe { task.as_mut().unwrap() };
    if task.state != TaskState::Finished {
        task.state = TaskState::Awakened;
    }
}
fn waker_wake_ref(data: *const ()) {
    waker_wake(data);
}
fn waker_drop(_data: *const ()) {}

static RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    waker_clone,
    waker_wake,
    waker_wake_ref,
    waker_drop,
);

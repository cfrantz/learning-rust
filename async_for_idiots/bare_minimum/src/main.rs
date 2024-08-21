// A demonstaration of the bare-minimum use of async.
//
// In this program, we will have 2 cooperating tasks which
// yield to each other.
#![feature(noop_waker)]

// These types exist in `core` and could be imported from there for a `no_std` program.
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

// We need a way to yield control.  We do this by returning a `Future` that
// reports `Pending` the first time, then `Ready`.
//
// Since `yield` is a keyword in rust for writing iterator/generator functions,
// we'll name ours `Suspend`.

struct Suspend {
    ready: bool,
}

impl Suspend {
    pub fn now() -> Self {
        Self { ready: false }
    }
}

impl Future for Suspend {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.ready {
            false => {
                self.as_mut().ready = true;
                Poll::Pending
            }
            true => Poll::Ready(()),
        }
    }
}

// We'll define our two "threads" as `async` functions.
// The async keyword instructs the compiler to rewrite the functions as a
// state machine returning a Future.
// It rewrites `fn foo(args) -> T` as `fn foo(args) -> impl Future<Output=T>`.
// The returned Future from any `async fn` is a struct that contains all
// of the execution state of that function such that the function can
// suspend and resume at each of the state transition points in the function
// (ie: the points in the function that await other Futures).
async fn hello() {
    loop {
        println!("Hello world!");
        // The `await` represents a state transition point.  It evaluates the
        // Future returned by Suspend::now() and returns if Pending.  When it
        // evaluates to Ready, execution resumes after the transition point.
        Suspend::now().await;
    }
}

async fn goodbye() {
    loop {
        println!("Goodbye world!");
        Suspend::now().await;
    }
}

// In our `main` function, we'll prove the most minimal possible executor for our
// async functions.

fn main() {
    // We'll create a async context which normally gives access to the waker.
    // Since we don't care about proper sleep/wake primitives for this super
    // primitive demonstration, we'll just create the no-op waker.
    let mut context = Context::from_waker(Waker::noop());

    // Our task queue is just an array of tasks.  We'll use the `pin!` macro
    // to ensure the Futures for our tasks can't be moved.
    let mut tasks: [Pin<&mut dyn Future<Output = ()>>; 2] = [
        std::pin::pin!(hello()),
        std::pin::pin!(goodbye()),
    ];

    // Our executor simply iterates over the list of tasks and polls each one.
    loop {
        for (i, t) in tasks.iter_mut().enumerate() {
            let r = t.as_mut().poll(&mut context);
            // In this example, our "threads" never exit, so the status
            // of their Futures is always `Pending`.
            println!("Executor: task={i} is {r:?}");
        }
    }
}

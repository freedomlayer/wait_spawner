
# WaitSpawner

WaitSpawner allows to drive a Rust Futures executor until no more progress is
possible. 

Possible uses:

- Writing tests for your asynchronous code.
- Solving deadlocks in your asynchronous code.


## Example

```rust
#![feature(futures_api, async_await, await_macro, arbitrary_self_types)]
#![feature(generators)]

extern crate wait_spawner;

use std::sync::{Arc, Mutex};
use futures::channel::mpsc;
use futures::task::SpawnExt;
use futures::{StreamExt, SinkExt};
use futures::executor::ThreadPool;

use wait_spawner::WaitSpawner;

fn main() {
    let mut thread_pool = ThreadPool::new().unwrap();

    let mut wspawner = WaitSpawner::new(thread_pool.clone());
    let waiter = wspawner.wait();

    let (mut a_sender, mut b_receiver) = mpsc::channel::<u32>(0);
    let (mut b_sender, mut a_receiver) = mpsc::channel::<u32>(0);

    // We spawn two futures that have a conversation.
    // Spawn first future:
    wspawner.spawn(async move {
        await!(a_sender.send(0)).unwrap();
        assert_eq!(await!(a_receiver.next()).unwrap(), 1);
        await!(a_sender.send(2)).unwrap();
    }).unwrap();

    // A shared result value, used to make sure that the second future
    // has finished
    let arc_mutex_res = Arc::new(Mutex::new(false));
    let c_arc_mutex_res = Arc::clone(&arc_mutex_res);

    // Spawn second future:
    wspawner.spawn(async move {
        assert_eq!(await!(b_receiver.next()).unwrap(), 0);
        await!(b_sender.send(1)).unwrap();
        assert_eq!(await!(b_receiver.next()).unwrap(), 2);
        let mut res_guard = c_arc_mutex_res.lock().unwrap();
        *res_guard = true;
    }).unwrap();

    // Keep running until no more progress is possible
    thread_pool.run(waiter);

    // Make sure that the second future has finished:
    let res_guard = arc_mutex_res.lock().unwrap();
    assert!(*res_guard);
}
```

## How does WaitSpawner work?

WaitSpawner serves as a proxy over your spawner.
It intercepts the following events:

- Spawning a future.
- The beginning and end of a `poll()` invocation over any future that was spawned through the WaitSpawner proxy.
- A call to `wake()` on any `Waker`

WaitSpawner maintains:
- A set of all futures in progress.
- The current amount of in progress `poll()` invocations.

Spawning a future adds the future to the set.
The beginning of a `poll()` invocation removes a future from the set. A call to
`wake()` on the future's `Waker` will put the future back on the set.

On any ending of a `poll()` invocation we check if the two following conditions are satisfied:
- The set of futures in progress is empty
- There are no more polls in progress.

if the two conditions are met, WaitSpawner notifies that no more progress can
be made.


## Information collection

If you choose to, WaitSpawner can collect information about the spawn sites for
all the spawned futures and print it to the screen. This can be useful to debug
your asynchronous code.

To activate information collection, construct WaitSpawn as follows:

```rust
let mut wspawner = WaitSpawner::new(thread_pool.clone())
                                    .collect_info();

```

This is example of the produced output:

```
---------[poll_end]----------
onging_polls = 0

---------[poll_end]----------
onging_polls = 0
id = 0
caller_info = Some(CallerInfo { name: "wait_spawner::wait_spawner::tests::test_two_futures::h55a9ee6c5603873e", filename: "src/wait_spawner.rs", lineno: 438 })

---------[poll_end]----------
onging_polls = 0
id = 1
caller_info = Some(CallerInfo { name: "wait_spawner::wait_spawner::tests::test_two_futures::h55a9ee6c5603873e", filename: "src/wait_spawner.rs", lineno: 445 })

---------[poll_end]----------
onging_polls = 0
```

Note that a print occurs every time a `poll()` invocation ends.
Every print contains the following information:

- The amount of ongoing `poll()` invocations.
- A list of all futures in progress. For each future:
    - A unique id.
    - Information about the spawn site. (The code that called `spawn()` or `spawn_with_handle()`)


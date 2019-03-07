#![feature(futures_api, async_await, await_macro, arbitrary_self_types)]
#![feature(generators)]
#![feature(nll)]

#![deny(
    warnings
)]

mod caller_info;
mod wait_spawner;

pub use self::wait_spawner::WaitSpawner;

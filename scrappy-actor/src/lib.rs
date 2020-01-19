//! # scrappy_actor is a rust actors framework
//!
//! [Actors](https://scrappy_actor.github.io/scrappy_actor/scrappy_actor/trait.Actor.html) are
//! objects which encapsulate state and behavior, they communicate
//! exclusively by exchanging messages. scrappy_actor actors are implemented
//! on top of [Tokio](https://tokio.rs).  Multiple actors can run in
//! same thread. Actors can run in multiple threads using the
//! [`Arbiter`](struct.Arbiter.html) API. Actors exchange typed
//! messages.
//!
//! ## Documentation
//!
//! * [User Guide](https://scrappy_actor.rs/book/scrappy_actor/)
//! * [Chat on gitter](https://gitter.im/scrappy_actor/scrappy_actor)
//! * [GitHub repository](https://github.com/scrappy_actor/scrappy_actor)
//! * [Cargo package](https://crates.io/crates/scrappy_actor)
//! * Minimum supported Rust version: 1.39 or later
//!
//! ## Features
//!
//! * Async/Sync actors.
//! * Actor communication in a local/thread context.
//! * Using Futures for asynchronous message handling.
//! * HTTP1/HTTP2 support ([scrappy_actor-web](https://github.com/scrappy_actor/scrappy_actor-web))
//! * Actor supervision.
//! * Typed messages (No `Any` type). Generic messages are allowed.
//!
//! ## Package feature
//!
//! * `resolver` - enables dns resolver actor, `scrappy_actor::actors::resolver`
//!
//! ## Tokio runtime
//!
//! At the moment scrappy_actor uses
//! [`current_thread`](https://docs.rs/tokio/0.1.13/tokio/runtime/current_thread/index.html) runtime.
//!
//! While it provides minimum overhead, it has its own limits:
//!
//! - You cannot use tokio's async file I/O, as it relies on blocking calls that are not available
//! in `current_thread`
//! - `Stdin`, `Stderr` and `Stdout` from `tokio::io` are the same as file I/O in that regard and
//! cannot be used in asynchronous manner in scrappy_actor.
pub use scrappy_derive::*;

mod actor;
mod context;
mod contextimpl;
mod contextitems;
mod handler;
mod stream;
mod supervisor;

mod address;
mod mailbox;

pub mod actors;
pub mod clock;
pub mod fut;
pub mod io;
pub mod registry;
pub mod sync;
pub mod utils;

pub use scrappy_rt::{Arbiter, System, SystemRunner};

pub use crate::actor::{
    Actor, ActorContext, ActorState, AsyncContext, Running, SpawnHandle, Supervised,
};
pub use crate::address::{Addr, MailboxError, Recipient, WeakAddr};
// pub use crate::arbiter::{Arbiter, ArbiterBuilder};
pub use crate::context::Context;
pub use crate::fut::{ActorFuture, ActorStream, FinishStream, WrapFuture, WrapStream};
pub use crate::handler::{
    ActorResponse, Handler, Message, MessageResult, Response, ResponseActFuture,
    ResponseFuture,
};
pub use crate::registry::{ArbiterService, Registry, SystemRegistry, SystemService};
pub use crate::stream::StreamHandler;
pub use crate::supervisor::Supervisor;
pub use crate::sync::{SyncArbiter, SyncContext};

#[doc(hidden)]
pub use crate::context::ContextFutureSpawner;

pub mod prelude {
    //! The `scrappy_actor` prelude.
    //!
    //! The purpose of this module is to alleviate imports of many common scrappy_actor
    //! traits by adding a glob import to the top of scrappy_actor heavy modules:
    //!
    //! ```
    //! # #![allow(unused_imports)]
    //! use scrappy_actor::prelude::*;
    //! ```

    #[doc(hidden)]
    pub use scrappy_derive::*;
    pub use scrappy_rt::{Arbiter, System, SystemRunner};

    pub use crate::actor::{
        Actor, ActorContext, ActorState, AsyncContext, Running, SpawnHandle, Supervised,
    };
    pub use crate::address::{
        Addr, MailboxError, Recipient, RecipientRequest, Request, SendError,
    };
    pub use crate::context::{Context, ContextFutureSpawner};
    pub use crate::fut::{ActorFuture, ActorStream, WrapFuture, WrapStream};
    pub use crate::handler::{
        ActorResponse, Handler, Message, MessageResult, Response, ResponseActFuture,
        ResponseFuture,
    };
    pub use crate::registry::{ArbiterService, SystemService};
    pub use crate::stream::StreamHandler;
    pub use crate::supervisor::Supervisor;
    pub use crate::sync::{SyncArbiter, SyncContext};

    pub use crate::actors;
    pub use crate::dev;
    pub use crate::fut;
    pub use crate::io;
    pub use crate::utils::{Condition, IntervalFunc, TimerFunc};

    pub use futures::{Future, Stream};
}

pub mod dev {
    //! The `scrappy_actor` prelude for library developers.
    //!
    //! The purpose of this module is to alleviate imports of many common scrappy_actor
    //! traits by adding a glob import to the top of scrappy_actor heavy modules:
    //!
    //! ```
    //! # #![allow(unused_imports)]
    //! use scrappy_actor::dev::*;
    //! ```

    pub use crate::prelude::*;

    pub use crate::address::{
        Envelope, EnvelopeProxy, RecipientRequest, Request, ToEnvelope,
    };
    pub mod channel {
        pub use crate::address::channel::{channel, AddressReceiver, AddressSender};
    }
    pub use crate::contextimpl::{AsyncContextParts, ContextFut, ContextParts};
    pub use crate::handler::{MessageResponse, ResponseChannel};
    pub use crate::mailbox::Mailbox;
    pub use crate::registry::{Registry, SystemRegistry};
}

/// Starts the system and executes the supplied future.
///
/// This function does the following:
///
/// * Creates and starts the scrappy_actor system with default configuration.
/// * Spawns the given future onto the current arbiter.
/// * Blocks the current thread until the system shuts down.
///
/// The `run` function returns when the `System::current().stop()`
/// method gets called.
///
/// # Examples
///
/// ```
/// use std::time::{Duration, Instant};
/// use tokio::time::delay_for;
///
/// fn main() {
///   scrappy_actor::run(async move {
///       delay_for(Duration::from_millis(100)).await;
///       scrappy_actor::System::current().stop();
///   });
/// }
/// ```
///
/// # Panics
///
/// This function panics if the scrappy_actor system is already running.
pub fn run<R>(f: R) -> std::io::Result<()>
where
    R: futures::Future<Output = ()> + 'static,
{
    Ok(scrappy_rt::System::new("Default").block_on(f))
}

/// Spawns a future on the current arbiter.
///
/// # Panics
///
/// This function panics if the scrappy_actor system is not running.
pub fn spawn<F>(f: F)
where
    F: futures::Future<Output = ()> + 'static,
{
    scrappy_rt::spawn(f);
}

/// `InternalServerError` for `scrappy_actor::MailboxError`
#[cfg(feature = "http")]
impl scrappy_http::ResponseError for MailboxError {}

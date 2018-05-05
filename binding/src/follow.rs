use super::traits::*;
use super::notify_fn::*;

use futures::*;
use futures::task::{Context, Waker};

use std::sync::*;
use std::marker::PhantomData;

///
/// The state of the binding for a follow stream
/// 
#[derive(Copy, Clone)]
enum FollowState {
    Unchanged,
    Changed
}

///
/// Core data structures for a follow stream
/// 
struct FollowCore<TValue, Binding: Bound<TValue>> {
    /// Changed if the binidng value has changed, or Unchanged if it is not changed
    state: FollowState,

    /// What to notify when this item is changed
    notify: Option<Waker>,

    /// The binding that this is following
    binding: Binding,

    /// Value is stored in the binding
    value: PhantomData<TValue>
}

///
/// Stream that follows the values of a binding
/// 
pub struct FollowStream<TValue, Binding: Bound<TValue>> {
    /// The core of this future
    core: Arc<Mutex<FollowCore<TValue, Binding>>>,

    /// Lifetime of the watcher
    watcher: Box<Releasable>,
}

impl<TValue, Binding: Bound<TValue>> Stream for FollowStream<TValue, Binding> {
    type Item   = TValue;
    type Error  = ();

    fn poll_next(&mut self, ctxt: &mut Context) -> Result<Async<Option<TValue>>, ()> {
        let mut core = self.core.lock().unwrap();

        match core.state {
            FollowState::Unchanged => {
                // Wake this future when changed
                core.notify = Some(ctxt.waker().clone());
                Ok(Async::Pending)
            },

            FollowState::Changed => {
                // Value has changed since we were last notified: return the changed value
                core.state = FollowState::Unchanged;
                Ok(Async::Ready(Some(core.binding.get())))
            }
        }
    }
}

///
/// Creates a stream from a binding
/// 
pub fn follow<TValue: 'static+Send, Binding: 'static+Bound<TValue>>(binding: Binding) -> FollowStream<TValue, Binding> {
    // Generate the initial core
    let core = FollowCore {
        state:      FollowState::Changed,
        notify:     None,
        binding:    binding,
        value:      PhantomData
    };

    // Notify whenever the binding changes
    let core        = Arc::new(Mutex::new(core));
    let weak_core   = Arc::downgrade(&core);
    let watcher     = core.lock().unwrap().binding.when_changed(notify(move || {
        if let Some(core) = weak_core.upgrade() {
            let mut core = core.lock().unwrap();

            core.state = FollowState::Changed;

            if let Some(notify) = core.notify.take() {
                notify.wake()
            }
        }
    }));

    // Create the stream
    FollowStream {
        core:       core,
        watcher:    watcher
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;

    use futures::executor;
    use futures::stream;

    use std::thread;
    use std::time::Duration;

    #[test]
    fn follow_stream_has_initial_value() {
        let binding     = bind(1);
        let bind_ref    = BindRef::from(binding.clone());
        let mut stream  = executor::block_on_stream(follow(bind_ref));

        assert!(stream.next() == Some(Ok(1)));
    }

    #[test]
    fn follow_stream_updates() {
        let mut binding = bind(1);
        let bind_ref    = BindRef::from(binding.clone());
        let mut stream  = executor::block_on_stream(follow(bind_ref));

        assert!(stream.next() == Some(Ok(1)));
        binding.set(2);
        assert!(stream.next() == Some(Ok(2)));
    }

    ///
    /// Creates a non-blocking version of an existing stream (useful for tests where we need
    /// to see if a stream will block or not)
    /// 
    fn nonblocking_stream<I, E, S: 'static+Stream<Item=I, Error=E>>(stream: S) -> Box<Stream<Item=Async<I>, Error=E>> {
        use self::Async::*;

        let mut stream = stream;
        Box::new(stream::poll_fn(move |ctxt| match stream.poll_next(ctxt) {
            Ok(Ready(Some(s)))  => Ok(Ready(Some(Ready(s)))),
            Ok(Ready(None))     => Ok(Ready(None)),
            Ok(Pending)         => Ok(Ready(Some(Pending))),
            Err(e)              => Err(e)
        }))
    }

    #[test]
    fn stream_is_unready_after_first_read() {
        use self::Async::*;

        let binding     = bind(1);
        let bind_ref    = BindRef::from(binding.clone());
        let stream      = nonblocking_stream(follow(bind_ref));
        let mut stream  = executor::block_on_stream(stream);

        assert!(stream.next() == Some(Ok(Ready(1))));
        assert!(stream.next() == Some(Ok(Pending)));
    }

    #[test]
    fn stream_is_immediate_ready_after_write() {
        use self::Async::*;

        let mut binding = bind(1);
        let bind_ref    = BindRef::from(binding.clone());
        let mut stream  = executor::block_on_stream(nonblocking_stream(follow(bind_ref)));

        assert!(stream.next() == Some(Ok(Ready(1))));
        binding.set(2);
        assert!(stream.next() == Some(Ok(Ready(2))));
    }

    #[test]
    fn will_wake_when_binding_is_updated() {
        let mut binding = bind(1);
        let bind_ref    = BindRef::from(binding.clone());
        let mut stream  = executor::block_on_stream(follow(bind_ref));

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            binding.set(2);
        });

        assert!(stream.next() == Some(Ok(1)));
        assert!(stream.next() == Some(Ok(2)));
    }
}

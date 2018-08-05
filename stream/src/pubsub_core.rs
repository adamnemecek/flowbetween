use futures::task;
use futures::task::Task;

use std::sync::*;
use std::collections::{VecDeque, HashMap};

///
/// The shared publisher core, used when subscribers need to send messages to their publisher
/// 
pub (crate) struct PubCore<Message> {
    /// The subscribers to this publisher
    pub subscribers: HashMap<usize, Arc<Mutex<SubCore<Message>>>>,

    /// The maximum size of queue to allow in any one subscriber
    pub max_queue_size: usize,
}

///
/// The core shared structure between a publisher and subscriber
/// 
pub (crate) struct SubCore<Message> {
    /// Unique ID for the subscriber represented by this core
    pub id: usize,

    /// True while the publisher owning this core is alive
    pub published: bool,

    /// Messages ready to be sent to this core
    pub waiting: VecDeque<Message>,

    /// Notification task for when the 'waiting' queue becomes non-empty
    pub notify_waiting: Option<Task>,

    /// If the publisher is waiting on this subscriber, this is the notification to send
    pub notify_ready: Option<Task>
}

impl<Message: Clone> PubCore<Message> {
    ///
    /// Attempts to publish a message to all subscribers, returning the list of notifications that need to be generated
    /// if successful, or None if the message could not be sent
    /// 
    pub fn publish(&mut self, message: &Message) -> Option<Vec<Task>> {
        let max_queue_size = self.max_queue_size;
        
        // Lock all of the subscribers
        let mut subscribers = self.subscribers.values()
            .map(|subscriber| subscriber.lock().unwrap())
            .collect::<Vec<_>>();

        // All subscribers must have enough space (we do not queue the message if any subscribe cannot accept it)
        if subscribers.iter().any(|subscriber| subscriber.waiting.len() > max_queue_size) {
            // At least one subscriber has a full queue
            None
        } else {
            // Send to all of the subscribers
            subscribers.iter_mut().for_each(|subscriber| subscriber.waiting.push_back(message.clone()));

            // Claim all of the notifications
            let notifications = subscribers.iter_mut()
                .filter_map(|subscriber| subscriber.notify_waiting.take())
                .collect::<Vec<_>>();

            // Result is the notifications to fire
            Some(notifications)
        }
    }

    ///
    /// Checks this core for readiness. If the core is not ready, returns false and sets the 'notify_ready' task
    /// 
    pub fn ready(&mut self) -> bool {
        // The core is ready if there are currently no subscribers using > max_queue_size slots
        let max_queue_size = self.max_queue_size;

        // Collect the subscribers into one place
        let mut subscribers = self.subscribers.values()
            .map(|subscriber| subscriber.lock().unwrap())
            .collect::<Vec<_>>();

        // Determine if we're ready or not
        let mut ready = true;
        for subscriber in subscribers.iter_mut() {
            if subscriber.waiting.len() >= max_queue_size {
                // Not ready
                ready = false;

                // This subscriber needs to notify this task when it becomes ready
                subscriber.notify_ready = Some(task::current());
            } else {
                // This subscriber doesn't need to notify anyone when it becomes ready
                subscriber.notify_ready = None;
            }
        }

        ready
    }
}
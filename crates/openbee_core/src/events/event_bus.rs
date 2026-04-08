//! Type-erased event bus with both immediate and queued event dispatch.

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Marker trait for events that can be published on the event bus.
pub trait Event: Any + Send + Sync + 'static {}

/// Type alias for a boxed event handler function.
type HandlerFn = Box<dyn FnMut(&dyn Any) + Send>;

/// Unique identifier for a subscription, used for unsubscribing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

/// Subscription entry holding the handler and its ID.
struct Subscription {
    id: SubscriptionId,
    handler: HandlerFn,
}

/// A type-erased publish/subscribe event bus.
///
/// Supports two dispatch modes:
/// - **Immediate**: `publish()` calls all handlers synchronously.
/// - **Queued**: `enqueue()` stores events to be dispatched later via `process_pending()`.
pub struct EventBus {
    /// Handlers grouped by event TypeId.
    handlers: HashMap<TypeId, Vec<Subscription>>,
    /// Pending events for deferred dispatch.
    pending: Vec<(TypeId, Box<dyn Any + Send + Sync>)>,
    /// Counter for generating unique subscription IDs.
    next_id: u64,
}

impl EventBus {
    /// Create a new empty event bus.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            pending: Vec::new(),
            next_id: 0,
        }
    }

    /// Subscribe a handler for events of type `E`. Returns a subscription ID
    /// that can be used to unsubscribe later.
    pub fn subscribe<E: Event>(
        &mut self,
        mut handler: impl FnMut(&E) + Send + 'static,
    ) -> SubscriptionId {
        let id = SubscriptionId(self.next_id);
        self.next_id += 1;

        let type_id = TypeId::of::<E>();
        let wrapped: HandlerFn = Box::new(move |any: &dyn Any| {
            if let Some(event) = any.downcast_ref::<E>() {
                handler(event);
            }
        });

        self.handlers
            .entry(type_id)
            .or_default()
            .push(Subscription {
                id,
                handler: wrapped,
            });

        id
    }

    /// Unsubscribe a handler by its subscription ID.
    pub fn unsubscribe(&mut self, sub_id: SubscriptionId) {
        for subs in self.handlers.values_mut() {
            subs.retain(|s| s.id != sub_id);
        }
    }

    /// Publish an event immediately, calling all registered handlers synchronously.
    pub fn publish<E: Event>(&mut self, event: E) {
        let type_id = TypeId::of::<E>();
        if let Some(subs) = self.handlers.get_mut(&type_id) {
            for sub in subs.iter_mut() {
                (sub.handler)(&event);
            }
        }
    }

    /// Enqueue an event for deferred dispatch. Call `process_pending()` to dispatch.
    pub fn enqueue<E: Event>(&mut self, event: E) {
        let type_id = TypeId::of::<E>();
        self.pending.push((type_id, Box::new(event)));
    }

    /// Dispatch all pending (queued) events and clear the queue.
    pub fn process_pending(&mut self) {
        let events = std::mem::take(&mut self.pending);
        for (type_id, event) in events {
            if let Some(subs) = self.handlers.get_mut(&type_id) {
                for sub in subs.iter_mut() {
                    (sub.handler)(event.as_ref());
                }
            }
        }
    }

    /// Return the number of pending events.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Clear all pending events without dispatching them.
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    /// Remove all handlers and pending events.
    pub fn clear_all(&mut self) {
        self.handlers.clear();
        self.pending.clear();
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestEvent(i32);
    impl Event for TestEvent {}

    #[test]
    fn test_immediate_publish() {
        let mut bus = EventBus::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        let r = received.clone();
        bus.subscribe(move |e: &TestEvent| {
            r.lock().unwrap().push(e.0);
        });
        bus.publish(TestEvent(42));
        assert_eq!(*received.lock().unwrap(), vec![42]);
    }

    #[test]
    fn test_queued_publish() {
        let mut bus = EventBus::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        let r = received.clone();
        bus.subscribe(move |e: &TestEvent| {
            r.lock().unwrap().push(e.0);
        });
        bus.enqueue(TestEvent(1));
        bus.enqueue(TestEvent(2));
        assert!(received.lock().unwrap().is_empty());
        bus.process_pending();
        assert_eq!(*received.lock().unwrap(), vec![1, 2]);
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_traits::{TimerEvent, TimerEventRequest, TimerSchedulerMsg};
use std::cmp::{self, Ord};
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

pub struct TimerScheduler(BinaryHeap<ScheduledEvent>);

struct ScheduledEvent {
    request: TimerEventRequest,
    for_time: Instant,
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &ScheduledEvent) -> cmp::Ordering {
        self.for_time.cmp(&other.for_time).reverse()
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &ScheduledEvent) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ScheduledEvent {}
impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &ScheduledEvent) -> bool {
        self as *const ScheduledEvent == other as *const ScheduledEvent
    }
}

impl TimerScheduler {
    pub fn new() -> Self {
        TimerScheduler(BinaryHeap::<ScheduledEvent>::new())
    }

    /// Return a timeout corresponding to the earliest scheduled event, if any,
    /// and check timers.
    pub fn get_timeout(&mut self) -> Option<Duration> {
        self.check_timers();
        let now = Instant::now();
        match self.0.peek() {
            // Do not schedule a timeout.
            None => None,
            // Schedule a timeout for the earliest event.
            Some(event) => Some(event.for_time - now),
        }
    }

    /// Dispatch any events whose due time is past
    pub fn check_timers(&mut self) {
        let now = Instant::now();
        loop {
            match self.0.peek() {
                // Dispatch the event if its due time is past
                Some(event) if event.for_time <= now => {
                    let TimerEventRequest(ref sender, source, id, _) = event.request;
                    let _ = sender.send(TimerEvent(source, id));
                },
                // Otherwise, we're done dispatching events
                _ => break,
            }
            // Remove the event from the priority queue
            // (Note this only executes when the first event has been dispatched
            self.0.pop();
        }
    }

    /// Handle an incoming timer request,
    /// and check timers.
    pub fn handle_timer_request(&mut self, request: TimerSchedulerMsg) {
        let TimerEventRequest(_, _, _, delay) = request.0;
        let schedule = Instant::now() + Duration::from_millis(delay.get());
        let event = ScheduledEvent {
            request: request.0,
            for_time: schedule,
        };
        self.0.push(event);
        self.check_timers();
    }
}

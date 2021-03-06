//! inspired by EVERY_N_MILLIS in https://github.com/FastLED/FastLED/blob/master/lib8tion.h
use core::convert::Infallible;
use core::sync::atomic::{AtomicU32, Ordering};
use nb::block;

/// Keep track of the milliseconds since boot.
/// TODO: this is really just a counterr. we just happen to be incrementing it every 1ms here
/// TODO: function for setting this to a specific amount. We can use the oldest ELAPSED_MS on the network
#[derive(Default)]
pub struct ElapsedMs(AtomicU32);

impl ElapsedMs {
    /// Block for some milliseconds.
    pub fn block(&self, millis: u32) {
        let mut until = EveryNMillis::new(self, millis);

        until.block(self);
    }

    /// Increment the time. Call this every millisecond.
    /// This should probably be called inside an interrupt.
    #[inline(always)]
    pub fn increment(&self) {
        self.increment_by(1);
    }

    /// Increment the time by a configurable amount.
    /// This shuold probably be called if you ever disable interrupts.
    /// TODO: this isn't safe. we should probably disable interrupts or something
    #[inline(always)]
    #[cfg(feature = "thumbv6")]
    pub fn increment_by(&self, by: u32) {
        let x = self.0.load(Ordering::Relaxed);
        self.0.store(x + by, Ordering::Relaxed)
    }

    /// Increment the time by a configurable amount.
    /// This shuold probably be called if you ever disable interrupts.
    /// TODO: think more about doing this safely
    #[inline(always)]
    #[cfg(not(feature = "thumbv6"))]
    pub fn increment_by(&self, by: u32) {
        self.0.fetch_add(by, Ordering::Relaxed);
    }

    /// Load the current time in milliseconds since boot
    #[inline(always)]
    pub fn now(&self) -> u32 {
        self.0.load(Ordering::Relaxed)
    }
}

/// TODO: what should we call this?
/// TODO: is usize a good type? maybe usize?
/// TODO: maybe a custom type so that we can have hz, or ms, or seconds, or minutes, etc.
pub struct EveryNMillis {
    next_trigger: u32,
    period_ms: u32,
}

impl EveryNMillis {
    pub fn new(elapsed_ms: &ElapsedMs, period_ms: u32) -> Self {
        let now = elapsed_ms.now();

        Self {
            next_trigger: now + period_ms,
            period_ms,
        }
    }

    pub fn ready(&mut self, elapsed_ms: &ElapsedMs) -> nb::Result<u32, Infallible> {
        let now = elapsed_ms.now();

        if now < self.next_trigger {
            return Err(nb::Error::WouldBlock);
        }

        // TODO: what if this is late? should we delay less?
        self.next_trigger = now + self.period_ms;

        Ok(now)
    }

    /// Block until this is ready
    pub fn block(&mut self, elapsed_ms: &ElapsedMs) {
        block!(self.ready(elapsed_ms)).unwrap();
    }
}

// Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::io;
use std::ops::Deref;
use std::os::unix::io::{AsRawFd, RawFd};

use libc::{
    epoll_create1, epoll_ctl, epoll_event, epoll_wait, EPOLLERR, EPOLLIN, EPOLLOUT, EPOLLRDHUP,
    EPOLL_CLOEXEC, EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD,
};

use utils::syscall::SyscallReturnCode;

/// Refers to the operation to be performed on a file descriptor.
#[repr(i32)]
pub enum ControlOperation {
    /// Add a file descriptor to the interest list.
    Add = EPOLL_CTL_ADD,
    /// Change the settings associated with a file descriptor that is
    /// already in the interest list.
    Modify = EPOLL_CTL_MOD,
    /// Remove a file descriptor from the interest list.
    Delete = EPOLL_CTL_DEL,
}

/// The type of events we can monitor a file descriptor for.
#[repr(i32)]
pub enum EventType {
    /// The associated file descriptor is available for read operations.
    Read = EPOLLIN,
    /// The associated file descriptor is available for write operations.
    Write = EPOLLOUT,
    /// Error condition happened on the associated file descriptor.
    Error = EPOLLERR,
    /// This can be used to detect peer shutdown when using Edge Triggered monitoring.
    ReadHangUp = EPOLLRDHUP,
}

impl EventType {
    /// Check if `self` is an event that we are monitoring using `events` mask.
    pub fn has_match_in(self, events: u32) -> bool {
        self as u32 & events != 0
    }
}

/// This is a wrapper over 'libc::epoll_event'.
///
/// We are using `transparent` here to be super sure that this struct and its fields
/// have the same alignment as those from the `epoll_event` struct from C.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Event(epoll_event);

impl Deref for Event {
    type Target = epoll_event;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Event {
    /// Create a new epoll_event instance for a fd on which we initially don't want to
    /// monitor any event.
    pub fn empty(data: u64) -> Self {
        Event(epoll_event {
            events: 0u32,
            u64: data,
        })
    }

    /// Create a new epoll_event instance for a fd on which we want to monitor the events
    /// specified by `events`.
    pub fn new(events: u32, data: u64) -> Self {
        Event(epoll_event { events, u64: data })
    }

    pub fn events(&self) -> u32 {
        self.events
    }

    pub fn data(&self) -> u64 {
        self.u64
    }

    // Converts from underlying bit representation, unless that representation
    // contains bits that do not correspond to a defined flag.
    pub fn from_bits(events: u32) -> Option<u32> {
        let mask = EventType::Read as u32
            | EventType::Write as u32
            | EventType::ReadHangUp as u32
            | EventType::Error as u32;
        if events & !mask == 0 {
            return Some(events);
        }
        None
    }
}

/// Stores a file descriptor referring to an epoll instance.
#[derive(Clone, Copy, Debug)]
pub struct Epoll {
    epoll_fd: RawFd,
}

impl Epoll {
    /// Create a new epoll file descriptor.
    pub fn new() -> io::Result<Self> {
        let epoll_fd = SyscallReturnCode(unsafe { epoll_create1(EPOLL_CLOEXEC) }).into_result()?;
        Ok(Epoll { epoll_fd })
    }

    /// Wrapper for `libc::epoll_ctl`.
    ///
    /// This can be used for adding, modifying or removing a file descriptor in the
    /// interest list of the epoll instance.
    ///
    /// # Arguments
    ///
    /// * `operation` refers to the action to be performed on the file descriptor.
    /// * `fd` is the file descriptor on which we want to perform `operation`.
    /// * `events` refers to the events we want to monitor `fd` for.
    pub fn ctl(self, operation: ControlOperation, fd: RawFd, events: u32) -> io::Result<()> {
        let mut event = Event::new(events, fd as u64);
        // Safe because we give a valid epoll file descriptor, a valid file descriptor to watch,
        // as well as a valid epoll_event structure. We also check the return value.
        SyscallReturnCode(unsafe {
            epoll_ctl(
                self.epoll_fd,
                operation as i32,
                fd,
                &mut event as *mut _ as *mut epoll_event,
            )
        })
        .into_empty_result()
    }

    /// Wrapper for `libc::epoll_wait`.
    ///
    /// # Arguments
    ///
    /// * `max_events` is the maximum number of events that can happen in the interest list.
    /// * `timeout` specifies for how long the `epoll_wait` system call will block
    /// (measured in milliseconds).
    pub fn wait(self, max_events: usize, timeout: i32) -> io::Result<Vec<Event>> {
        let mut buffer = vec![Event::empty(0); max_events];
        // Safe because we give a valid epoll file descriptor and an array of epoll_event structures
        // that will be modified by the kernel to indicate information about the subset of file
        // descriptors in the interest list. We also check the return value.
        let events_count = SyscallReturnCode(unsafe {
            epoll_wait(
                self.epoll_fd,
                buffer.as_mut_slice() as *mut _ as *mut epoll_event,
                buffer.len() as i32,
                timeout,
            )
        })
        .into_result()? as usize;

        Ok(buffer[..events_count].to_vec())
    }
}

impl AsRawFd for Epoll {
    fn as_raw_fd(&self) -> RawFd {
        self.epoll_fd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_ops() {
        let mut event = Event::empty(1);
        assert_eq!(event.events(), 0);
        assert_eq!(event.data(), 1);

        event = Event::new(1, 2);
        assert_eq!(event.events(), 1);
        assert_eq!(event.data(), 2);
    }

    #[test]
    fn test_from_bits() {
        // The least significant bit set matches `EPOLLIN` event type.
        let mut events = Event::from_bits(1);
        assert_eq!(events.unwrap(), 1);
        // The third least significant bit set doesn't match any of the
        // epoll events that we expect.
        events = Event::from_bits(2);
        assert!(events.is_none());
    }

    #[test]
    fn test_epoll() {
        let epoll = Epoll::new().unwrap();
        assert_eq!(epoll.epoll_fd, epoll.as_raw_fd());
    }
}

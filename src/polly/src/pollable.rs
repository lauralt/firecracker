// Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::convert::From;
use std::fmt::Formatter;
use std::os::unix::io::RawFd;

use epoll;

pub type EventRegistrationData = (RawFd, EventSet);

pub enum PollableOp {
    /// Register a new handler for a pollable fd and a set of events.
    Register(EventRegistrationData),
    /// Unregister a handler for a pollable fd.
    Unregister(RawFd),
    /// Update the event set for a specified pollable fd.
    Update(EventRegistrationData),
}

impl std::fmt::Debug for PollableOp {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use self::PollableOp::*;

        match self {
            Register(data) => write!(f, "Register {:?}", data),
            Unregister(data) => write!(f, "Unregister {:?}", data),
            Update(data) => write!(f, "Update {:?}", data),
        }
    }
}

bitflags! {
    /// Contains the events we want to monitor a fd and it works as an interface between
    /// the platform specific events and some general events we are watching.
    pub struct EventSet: u8 {
        const NONE = 0b0000_0000;
        const READ = 0b0000_0001;
        const WRITE = 0b0000_0010;
        const CLOSE = 0b0000_0100;
    }
}

/// Wraps the epoll specific event mask interface.
impl EventSet {
    /// Check if this is a read event.
    pub fn is_readable(self) -> bool {
        self.contains(EventSet::READ)
    }
    /// Check if this is a write event.
    pub fn is_writeable(self) -> bool {
        self.contains(EventSet::WRITE)
    }
    /// Check if this is a close event.
    pub fn is_closed(self) -> bool {
        self.contains(EventSet::CLOSE)
    }
}

impl From<EventSet> for u32 {
    fn from(event: EventSet) -> u32 {
        let mut epoll_event_mask = 0u32;

        if event.is_readable() {
            epoll_event_mask |= epoll::EventType::Read as u32;
        }

        if event.is_writeable() {
            epoll_event_mask |= epoll::EventType::Write as u32;
        }

        if event.is_closed() {
            epoll_event_mask |= epoll::EventType::ReadHangUp as u32;
        }

        epoll_event_mask
    }
}

/// Associates the file descriptor represented by `fd` with the events
/// that the user is interested for it.
pub struct EpollConfig {
    fd: RawFd,
    event_mask: EventSet,
}

impl EpollConfig {
    /// Constructs a new EpollConfig for the specified fd.
    pub fn new(fd: RawFd) -> EpollConfig {
        EpollConfig {
            fd,
            event_mask: EventSet::NONE,
        }
    }

    /// Caller is interested in fd read events.
    pub fn readable(&mut self) -> &mut EpollConfig {
        self.event_mask |= EventSet::READ;
        self
    }

    /// Caller is interested in fd write events.
    pub fn writeable(&mut self) -> &mut EpollConfig {
        self.event_mask |= EventSet::WRITE;
        self
    }

    /// Caller is interested in fd close events.
    pub fn closeable(&mut self) -> &mut EpollConfig {
        self.event_mask |= EventSet::CLOSE;
        self
    }

    /// Create a `Register` PollableOp.
    pub fn register(&self) -> PollableOp {
        PollableOp::Register((self.fd, self.event_mask))
    }

    /// Create an `Unregister` PollableOp.
    pub fn unregister(&self) -> PollableOp {
        PollableOp::Unregister(self.fd)
    }

    /// Create an `Update` PollableOp.
    pub fn update(&self) -> PollableOp {
        PollableOp::Update((self.fd, self.event_mask))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::os::unix::io::AsRawFd;

    #[test]
    fn test_epoll_config() {
        let pollable = io::stdin().as_raw_fd();
        let mut op_register = EpollConfig::new(pollable)
            .readable()
            .writeable()
            .closeable()
            .register();
        assert_eq!(
            format!("{:?}", op_register),
            "Register (0, READ | WRITE | CLOSE)"
        );

        match op_register {
            PollableOp::Register(data) => {
                assert_eq!(data.0, pollable);
                assert_eq!(data.1, EventSet::READ | EventSet::WRITE | EventSet::CLOSE);
            }
            _ => panic!("Expected Register op"),
        }

        op_register = EpollConfig::new(pollable).closeable().unregister();

        match op_register {
            PollableOp::Unregister(data) => {
                assert_eq!(data, pollable);
            }
            _ => panic!("Expected Unregister op"),
        }

        op_register = EpollConfig::new(pollable).readable().update();

        match op_register {
            PollableOp::Update(data) => {
                assert_eq!(data.0, pollable);
                assert_eq!(data.1, EventSet::READ);
            }
            _ => panic!("Expected Update op"),
        }
    }
}

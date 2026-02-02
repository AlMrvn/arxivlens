use crate::app::AppResult;
use ratatui::crossterm::event::{
    self, Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent,
};
use std::sync::mpsc;
use std::thread;

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

/// Terminal event handler.
///
/// This struct manages a background thread that polls for crossterm events
/// and sends them through a channel to the main loop.
#[derive(Debug)]
pub struct EventHandler {
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
    /// Event handler thread handle.
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    ///
    /// Spawns a background thread to poll terminal events.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let handler = thread::spawn(move || {
            // This blocks the thread until an event occurs, looping as long as read() is Ok
            while let Ok(crossterm_event) = event::read() {
                let result = match crossterm_event {
                    CrosstermEvent::Key(e) => {
                        if e.kind == KeyEventKind::Press {
                            sender.send(Event::Key(e))
                        } else {
                            Ok(())
                        }
                    }
                    CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),
                    CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                    CrosstermEvent::FocusGained
                    | CrosstermEvent::FocusLost
                    | CrosstermEvent::Paste(_) => Ok(()),
                };

                // If the receiver was dropped (app closing), break the loop
                if result.is_err() {
                    break;
                }
            }
        });

        Self { receiver, handler }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function blocks the current thread until an event is available.
    pub fn next(&self) -> AppResult<Event> {
        Ok(self.receiver.recv()?)
    }
}

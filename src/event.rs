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
#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
    /// Event handler thread.
    handler: thread::JoinHandle<()>,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || loop {
                match event::read().expect("unable to read event") {
                    CrosstermEvent::Key(e) => {
                        if e.kind == KeyEventKind::Press {
                            sender.send(Event::Key(e))
                        } else {
                            Ok(())
                        }
                    }
                    CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),
                    CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                    CrosstermEvent::FocusGained => Ok(()),
                    CrosstermEvent::FocusLost => Ok(()),
                    CrosstermEvent::Paste(_) => unimplemented!(),
                }
                .expect("failed to send terminal event")
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub fn next(&self) -> AppResult<Event> {
        Ok(self.receiver.recv()?)
    }
}

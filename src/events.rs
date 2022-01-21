use crossterm::event::{self, Event as CEvent, KeyEvent};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub enum AppEvent<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<AppEvent<KeyEvent>>,
}

impl Events {
    // a lot of this code comes from these two sources:
    // https://github.com/deepu105/battleship-rs/blob/main/src/event.rs
    // https://github.com/zupzup/rust-commandline-example/blob/main/src/main.rs
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut last_tick = Instant::now();

            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("poll works") {
                    if let CEvent::Key(key) = event::read().expect("can read events") {
                        tx.send(AppEvent::Input(key)).expect("can send events");
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = tx.send(AppEvent::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        Events { rx: rx }
    }

    pub fn next(&self) -> Result<AppEvent<KeyEvent>, mpsc::RecvError> {
        self.rx.recv()
    }
}

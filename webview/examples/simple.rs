use std::{
    sync::mpsc::{channel, Sender},
    thread,
    time::Duration,
};

use minifb::{MouseButton, MouseMode, Window, WindowOptions};
use webview::{
    execute_subprocess, is_subprocess, ActionState, App, AppObserver, AppOptions, MouseAction,
    MouseButtons, PageObserver, PageOptions, Position,
};

struct ImplPageObserver {
    sender: Sender<Vec<u8>>,
}

impl PageObserver for ImplPageObserver {
    fn on_frame(&self, buf: &[u8], _: u32, _: u32) {
        self.sender.send(buf.to_vec()).unwrap();
    }
}

struct ImplAppObserver;

impl AppObserver for ImplAppObserver {}

fn run_cef() -> anyhow::Result<()> {
    let (sender, receiver) = channel();
    let app = App::new(&AppOptions::default(), ImplAppObserver).unwrap();

    let settings = PageOptions::default();

    let browser = app
        .create_page("https://google.com", &settings, ImplPageObserver { sender })
        .unwrap();
    thread::spawn(move || {
        let mut window = Window::new(
            "simple",
            settings.width as usize,
            settings.height as usize,
            WindowOptions::default(),
        )?;

        window.limit_update_rate(Some(Duration::from_millis(
            1000 / settings.windowless_frame_rate as u64,
        )));

        let mut frame = vec![0u8; (settings.width * settings.height * 4) as usize];
        loop {
            if let Some((x, y)) = window
                .get_mouse_pos(MouseMode::Clamp)
                .map(|(x, y)| (x as i32, y as i32))
            {
                if window.get_mouse_down(MouseButton::Left) {
                    browser.mouse(MouseAction::Click(
                        MouseButtons::kLeft,
                        ActionState::Down,
                        Some(Position { x, y }),
                    ));

                    browser.mouse(MouseAction::Click(
                        MouseButtons::kLeft,
                        ActionState::Up,
                        None,
                    ));
                }
            }

            if let Ok(f) = receiver.try_recv() {
                frame = f;
            }

            let (_, shorts, _) = unsafe { frame.align_to::<u32>() };
            window.update_with_buffer(shorts, settings.width as usize, settings.height as usize)?;
            thread::sleep(Duration::from_millis(
                1000 / settings.windowless_frame_rate as u64,
            ));
        }

        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

fn main() -> anyhow::Result<()> {
    if is_subprocess() {
        execute_subprocess().unwrap();
    }

    run_cef()?;
    Ok(())
}

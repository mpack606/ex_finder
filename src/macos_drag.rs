use iced::window::Window;
use std::path::PathBuf;
use std::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Outcome {
    pub dropped: bool,
    pub position: Option<iced::Point>,
}

#[cfg(target_os = "macos")]
pub fn start(window: &dyn Window, paths: &[PathBuf]) -> Result<mpsc::Receiver<Outcome>, String> {
    use iced::window::raw_window_handle::RawWindowHandle;

    let Some(first_path) = paths.first() else {
        return Err("There are no files to drag".to_owned());
    };
    let preview = crate::app_icons::get_app_icon_for_file(first_path)
        .ok_or_else(|| "Could not load a file icon for dragging".to_owned())?;
    let (sender, receiver) = mpsc::channel();
    let sender = std::sync::Mutex::new(Some(sender));
    let handle = window
        .window_handle()
        .map_err(|error| format!("Could not access the app window: {error}"))?;
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return Err("Could not access the macOS app window".to_owned());
    };
    let view_address = handle.ns_view.as_ptr() as usize;

    drag::start_drag(
        &window,
        drag::DragItem::Files(paths.to_vec()),
        drag::Image::Raw(preview),
        move |result, _| {
            let outcome = Outcome {
                dropped: matches!(result, drag::DragResult::Dropped),
                // The drag crate invokes this callback on AppKit's main thread while
                // the source window is still alive.
                position: unsafe { cursor_position_from_view(view_address) },
            };
            if let Ok(mut sender) = sender.lock()
                && let Some(sender) = sender.take()
            {
                let _ = sender.send(outcome);
            }
        },
        drag::Options {
            mode: drag::DragMode::Copy,
            ..Default::default()
        },
    )
    .map_err(|error| format!("Could not start file dragging: {error}"))?;

    Ok(receiver)
}

#[cfg(not(target_os = "macos"))]
pub fn start(_window: &dyn Window, _paths: &[PathBuf]) -> Result<mpsc::Receiver<Outcome>, String> {
    Err("Dragging files outside the app is only available on macOS".to_owned())
}

#[cfg(target_os = "macos")]
unsafe fn cursor_position_from_view(view_address: usize) -> Option<iced::Point> {
    use objc2_app_kit::NSView;

    // SAFETY: The caller guarantees this is the source window's live NSView and
    // that this function runs on AppKit's main thread.
    let view = unsafe { &*(view_address as *const NSView) };
    let native_window = view.window()?;
    let point = view.convertPoint_fromView(native_window.mouseLocationOutsideOfEventStream(), None);
    let bounds = view.bounds();
    let y = if view.isFlipped() {
        point.y - bounds.origin.y
    } else {
        bounds.size.height - (point.y - bounds.origin.y)
    };

    Some(iced::Point::new(
        (point.x - bounds.origin.x) as f32,
        y as f32,
    ))
}

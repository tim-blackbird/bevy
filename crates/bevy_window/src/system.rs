use crate::{Window, WindowCloseRequested, WindowId, WindowMode, Windows};

use bevy_app::AppExit;
use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, Input};
use bevy_utils::HashMap;

/// Exit the application when there are no open windows.
///
/// This system is added by the [`WindowPlugin`] in the default configuration.
/// To disable this behaviour, set `close_when_requested` (on the [`WindowPlugin`]) to `false`.
/// Ensure that you read the caveats documented on that field if doing so.
///
/// [`WindowPlugin`]: crate::WindowPlugin
pub fn exit_on_all_closed(mut app_exit_events: EventWriter<AppExit>, windows: Res<Windows>) {
    if windows.iter().count() == 0 {
        app_exit_events.send(AppExit);
    }
}

/// Close windows in response to [`WindowCloseRequested`] (e.g.  when the close button is pressed).
///
/// This system is added by the [`WindowPlugin`] in the default configuration.
/// To disable this behaviour, set `close_when_requested` (on the [`WindowPlugin`]) to `false`.
/// Ensure that you read the caveats documented on that field if doing so.
///
/// [`WindowPlugin`]: crate::WindowPlugin
pub fn close_when_requested(
    mut windows: ResMut<Windows>,
    mut closed: EventReader<WindowCloseRequested>,
) {
    for event in closed.iter() {
        windows.get_mut(event.id).map(Window::close);
    }
}

/// Close the focused window whenever the escape key (<kbd>Esc</kbd>) is pressed
///
/// This is useful for examples or prototyping.
pub fn close_on_esc(mut windows: ResMut<Windows>, input: Res<Input<KeyCode>>) {
    if let Some(window) = windows.get_focused_mut() {
        if input.just_pressed(KeyCode::Escape) {
            window.close();
        }
    }
}

/// Toggle between the last selected fullscreen mode and windowed mode on the focused window whenever `Alt + Enter` is pressed.
pub fn toggle_fullscreen_shortcut(
    mut windows: ResMut<Windows>,
    input: Res<Input<KeyCode>>,
    mut last_fullscreen_mode: Local<HashMap<WindowId, WindowMode>>,
) {
    if windows.is_changed() {
        for window in windows.iter() {
            if window.mode() != WindowMode::Windowed {
                // Update since a fullscreen mode might have been set outside of this system.
                last_fullscreen_mode
                    .entry(window.id())
                    .insert(window.mode());
            }
        }

        // Remove entries from closed windows.
        last_fullscreen_mode.retain(|id, _| windows.get(*id).is_some());
    }

    if let Some(window) = windows.get_focused_mut() {
        if input.any_pressed([KeyCode::LAlt, KeyCode::RAlt]) && input.just_pressed(KeyCode::Return)
        {
            if window.mode() == WindowMode::Windowed {
                let mode = *last_fullscreen_mode
                    .entry(window.id())
                    .or_insert(WindowMode::Fullscreen);

                window.set_mode(mode);
            } else {
                window.set_mode(WindowMode::Windowed);
            }
        }
    }
}

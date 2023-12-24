//! Basic input resource

use std::collections::BTreeMap;

use winit::event::WindowEvent;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ButtonState {
    Pressed,
    Released,
    Down,
    Up,
}

impl ButtonState {
    /// Returns true if the key is down
    pub fn is_down(self) -> bool {
        self == ButtonState::Pressed || self == ButtonState::Down
    }
}

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub enum Button {
    Key(u32),
    Mouse(u32),
}

pub use Button::*;

#[derive(Default)]
pub struct Input {
    buttons: BTreeMap<Button, ButtonState>,
}

impl Input {
    /// Get the current state of a button
    pub fn button<B: Into<Button>>(&self, button: B) -> ButtonState {
        *self.buttons.get(&button.into()).unwrap_or(&ButtonState::Up)
    }

    pub fn push_button(&mut self, button: Button) {
        self.buttons
            .entry(button)
            .and_modify(|state| {
                // If the button is up then set it to pressed
                if !state.is_down() {
                    *state = ButtonState::Pressed;
                }
            })
            .or_insert(ButtonState::Pressed);
    }

    pub fn release_button(&mut self, button: Button) {
        self.buttons.insert(button, ButtonState::Released);
    }

    /// Send a winit event into this input tracker
    pub(crate) fn recv_winit(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                let key = Button::Key(input.scancode);
                match input.state {
                    winit::event::ElementState::Pressed => {
                        self.push_button(key);
                    }
                    winit::event::ElementState::Released => {
                        self.release_button(key);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let key = Button::Mouse(match button {
                    winit::event::MouseButton::Left => 0,
                    winit::event::MouseButton::Right => 1,
                    winit::event::MouseButton::Middle => 2,
                    winit::event::MouseButton::Other(i) => *i as u32,
                });

                match state {
                    winit::event::ElementState::Pressed => self.push_button(key),
                    winit::event::ElementState::Released => self.release_button(key),
                }
            }
            _ => {}
        }
    }

    /// update release/press events to a held input
    fn tick(&mut self) {
        for (_btn, state) in self.buttons.iter_mut() {
            match state {
                ButtonState::Pressed => *state = ButtonState::Down,
                ButtonState::Released => *state = ButtonState::Up,
                _ => {}
            }
        }
    }
}

pub mod font;
pub mod gl_context;

use gl_context::GLContext;
use glium::glutin;
use glutin::dpi::{LogicalPosition, LogicalSize};

pub enum AppState {
    On,
    Off,
}

pub trait SingleWindowApp {
    type ViewUpdate;
    type State;

    fn render(&self, display: &mut glium::Display);

    fn receive(&mut self, message: Self::ViewUpdate);

    fn press_key(&mut self, virtual_key: glium::glutin::event::VirtualKeyCode);

    fn set_focus(&mut self, focus: bool);

    fn move_cursor(&mut self, new_position: LogicalPosition<f64>);

    fn press_mouse(&mut self, button: glutin::event::MouseButton);

    fn release_mouse(&mut self, button: glutin::event::MouseButton);

    fn change_modifiers(&mut self, modifiers: glutin::event::ModifiersState);

    fn resize(&mut self, new_size: LogicalSize<f64>);

    fn release_key(&mut self, virtual_key: glium::glutin::event::VirtualKeyCode);

    fn initial_state() -> Self::State;

    fn manage_state<U: Fn(Self::ViewUpdate) -> AppState>(
        state: &mut Self::State,
        update_view: U,
    ) -> !;

    fn run_with(&'static mut self, gl_context: GLContext<Self::ViewUpdate>) -> !
    where
        Self: Sized + 'static,
        Self::ViewUpdate: Send + std::fmt::Debug, // This truly is a debug thing, should not be here for release
    {
        let GLContext {
            event_loop,
            mut display,
            starting_scale_factor,
        } = gl_context;
        let mut scale_factor = starting_scale_factor;

        let event_loop_proxy = event_loop.create_proxy();

        std::thread::spawn(move || {
            Self::manage_state(&mut Self::initial_state(), |event| {
                match event_loop_proxy.clone().send_event(event) {
                    Ok(()) => AppState::On,
                    Err(_) => AppState::Off,
                }
            })
        });

        event_loop.run(move |event, _event_loop_window_target, control_flow| {
            // Rendering
            self.render(&mut display);

            // By default, just wait until the next frame to render
            let next_frame_time =
                std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
            *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

            // Event processing
            match event {
                glutin::event::Event::WindowEvent {
                    event: window_event,
                    ..
                } => match window_event {
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    glutin::event::WindowEvent::Destroyed => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(virtual_key) = input.virtual_keycode {
                            match input.state {
                                glutin::event::ElementState::Pressed => self.press_key(virtual_key),
                                glutin::event::ElementState::Released => {
                                    self.release_key(virtual_key)
                                }
                            }
                            self.press_key(virtual_key);
                        }
                    }
                    glutin::event::WindowEvent::Focused(b) => {
                        self.set_focus(b);
                    }
                    glutin::event::WindowEvent::Resized(new_physical_size) => {
                        self.resize(LogicalSize::from_physical(new_physical_size, scale_factor));
                    }
                    glutin::event::WindowEvent::CursorMoved {
                        position: physical_position,
                        ..
                    } => {
                        self.move_cursor(LogicalPosition::from_physical(
                            physical_position,
                            scale_factor,
                        ));
                    }
                    glutin::event::WindowEvent::ModifiersChanged(new_modifiers) => {
                        self.change_modifiers(new_modifiers);
                    }
                    glutin::event::WindowEvent::MouseInput { state, button, .. } => match state {
                        glutin::event::ElementState::Pressed => {
                            self.press_mouse(button);
                        }
                        glutin::event::ElementState::Released => {
                            self.release_mouse(button);
                        }
                    },
                    glutin::event::WindowEvent::ScaleFactorChanged {
                        scale_factor: new_scale_factor,
                        ..
                    } => {
                        scale_factor = new_scale_factor;
                    }
                    other_window_event => {
                        // For debugging, delete before release
                        println!("Other window event: {:?}", other_window_event);
                    }
                },
                other_glutin_event => {
                    // For debugging, delete before release
                    println!("Other glutin event: {:?}", other_glutin_event);
                }
            }
        });
    }
}

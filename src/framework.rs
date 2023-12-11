use crate::{
    game::game_main,
    managers::{
        render,
        sound::{self, set_listener_transform}, systems, input, networking,
    },
};
use glium::{glutin::{ContextBuilder, event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, event::WindowEvent}, Display, backend::glutin};
use std::time::Instant;

pub fn start_game(debug_mode: DebugMode) {
    let event_loop = EventLoop::new();
    let mut display: Option<Display>;
    match networking::get_current_networking_mode() {
        networking::NetworkingMode::Server(_) => display = None,
        _ => {
            let wb = WindowBuilder::new();
            let cb = ContextBuilder::new().with_srgb(false);
            display = Some(Display::new(wb, cb, &event_loop).expect("failed to create glium display"));
        },
    }

    let mut frames_count: usize = 0;
    let mut now = std::time::Instant::now();
    let mut last_frame = std::time::Instant::now();

    match networking::get_current_networking_mode() {
        networking::NetworkingMode::Server(_) => (),
        _ => sound::init().unwrap(),
    };

    game_main::start();

    let mut win_w = 0;
    let mut win_h = 0;

    event_loop.run(move |ev, _, control_flow| {
        match ev {
            glium::glutin::event::Event::MainEventsCleared => {
                game_main::update();
                systems::update();
                input::update();
                networking::update(Instant::now().duration_since(last_frame));

                match networking::get_current_networking_mode() {
                    networking::NetworkingMode::Server(_) => (),
                    _ => {
                        set_listener_transform(render::get_camera_position(), render::get_camera_front());
                        win_w = display.as_ref().expect("display is none(should be only in server mode)").gl_window().window().inner_size().width;
                        win_h = display.as_ref().expect("display is none(should be only in server mode)").gl_window().window().inner_size().height;
                        unsafe {
                            render::ASPECT_RATIO = win_w as f32 / win_h as f32;
                        }

                        let mut target = display.as_ref().expect("display is none(should be only in server mode)").draw();

                        render::draw(&mut target);
                        game_main::render();
                        systems::render(&mut display.as_mut().expect("display is none(should be only in server mode)"), &mut target);

                        target.finish().unwrap();
                    }
                }

                last_frame = Instant::now();
            }
            glutin::glutin::event::Event::WindowEvent { event, .. } => {
                input::reg_event(&event);
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                }
            }
            _ => (),
        }

        match debug_mode {
            DebugMode::None => (),
            _ => {
                match networking::get_current_networking_mode() {
                    networking::NetworkingMode::Server(_) => (),
                    _ => {
                        let fps = get_fps(&now, &frames_count);
                        if fps.is_some() {
                            let fps = fps.unwrap();
                            display
                                .as_ref()
                                .expect("display is none(should be only in server mode)")
                                .gl_window()
                                .window()
                                .set_title(&format!("very cool window: {fps} fps"));
                            frames_count = 0;
                            now = Instant::now();
                        }
                    }
                }
            }
        }

        frames_count += 1;
    });
}

fn get_fps(now: &Instant, frames: &usize) -> Option<usize> {
    let one_second = std::time::Duration::new(1, 0);

    if now.elapsed() > one_second {
        return Some(frames.clone());
    }
    None
}

#[derive(Clone, Copy)]
pub enum DebugMode {
    None,
    ShowFps,
    Full,
}

fn set_audio_listener_transformations() {
    let camera_pos = render::get_camera_position();
    let camera_rot = render::get_camera_rotation();

    sound::set_listener_position(camera_pos);
    sound::set_listener_orientation(camera_rot);
}

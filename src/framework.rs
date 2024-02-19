use crate::{
    game::game_main,
    managers::{
        render,
        sound::{self, set_listener_transform}, systems, input, networking::{self, NetworkingMode}, physics,
    },
};
use glium::{glutin::{ContextBuilder, event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, event::WindowEvent}, Display, backend::glutin};
use std::{time::{Instant, Duration}, num::NonZeroU32};

static mut DEBUG_MODE: DebugMode = DebugMode::None;
static mut DELTA_TIME: Duration = Duration::new(0, 0);

pub fn start_game_with_render(debug_mode: DebugMode) {
    unsafe { DEBUG_MODE = debug_mode } 

    let event_loop = EventLoop::new();
    let mut display: Option<Display>;
    match networking::get_current_networking_mode() {
        networking::NetworkingMode::Server(_) => {
            display = None;
        },
        _ => {
            let wb = WindowBuilder::new().with_title("projectbaldej").with_transparent(false);
            let cb = ContextBuilder::new().with_srgb(false).with_vsync(true);
            display = Some(Display::new(wb, cb, &event_loop).expect("failed to create glium display"));
        },
    }

    let mut frames_count: usize = 0;
    let mut now = std::time::Instant::now();
    let mut last_frame = std::time::Instant::now();

    match networking::get_current_networking_mode() {
        networking::NetworkingMode::Server(_) => (),
        _ => {
            sound::init().unwrap();
            render::init(display.as_ref().unwrap());
        },
    };

    game_main::start();

    let mut win_w = 0;
    let mut win_h = 0;


    let frame_time = Duration::from_millis(16);


    event_loop.run(move |ev, _, control_flow| {
        let time_since_last_frame = last_frame.elapsed();
        update_game(time_since_last_frame);

        if let NetworkingMode::Server(_) = networking::get_current_networking_mode() {
            if time_since_last_frame < frame_time {
                let wait_time = frame_time - time_since_last_frame;
                let wait_until = Instant::now() + wait_time;
                *control_flow = ControlFlow::WaitUntil(wait_until);
            }
            last_frame = Instant::now();
        }

        match ev {
            glium::glutin::event::Event::MainEventsCleared => {
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
                        render::debug_draw(display.as_ref().expect("display is none(should be only in server mode)"), &mut target);

                        target.finish().unwrap();
                        last_frame = Instant::now();
                    }
                }
            }
            glutin::glutin::event::Event::WindowEvent { event, .. } => {
                input::reg_event(&event);
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                        networking::disconnect();
                    },
                    _ => (),
                }
            }
            _ => (),
        }

        match debug_mode {
            DebugMode::None => (),
            _ => {
                match networking::get_current_networking_mode() {
                    networking::NetworkingMode::Server(_) => {
                        let fps = get_fps(&now, &frames_count);
                        if fps.is_some() {
                            let _ = fps.unwrap();
                            //println!("fps: {}", fps);
                            frames_count = 0;
                            now = Instant::now();
                        }
                    },
                    _ => {
                        let fps = get_fps(&now, &frames_count);
                        if fps.is_some() {
                            let fps = fps.unwrap();
                            display
                                .as_ref()
                                .expect("display is none(should be only in server mode)")
                                .gl_window()
                                .window()
                                .set_title(&format!("projectbaldej: {fps} fps"));
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

pub fn start_game_without_render() {
    println!("starting game without render");
    game_main::start();

    let tickrate_tick = Duration::from_millis(16);
    let clock = chron::Clock::new(NonZeroU32::new(60).unwrap());

    for tick in clock {
        match tick {
            chron::clock::Tick::Update => {
                update_game(tickrate_tick);
            },
            chron::clock::Tick::Render { interpolation: _ } => { }
        }
    }
}

fn update_game(delta_time: Duration) {
    set_delta_time(delta_time);
    physics::update();
    input::update();
    networking::update(delta_time);
    game_main::update();
    systems::update();
}

fn get_fps(now: &Instant, frames: &usize) -> Option<usize> {
    let one_second = std::time::Duration::new(1, 0);

    if now.elapsed() > one_second {
        return Some(frames.clone());
    }
    None
}

pub fn get_debug_mode() -> DebugMode {
    unsafe { DEBUG_MODE }
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

fn set_delta_time(dt: Duration) {
    unsafe {
        DELTA_TIME = dt;
    }
}

pub fn get_delta_time() -> Duration {
    unsafe { DELTA_TIME }
}

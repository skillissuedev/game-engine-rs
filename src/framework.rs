use std::{time::Instant, thread};
use conrod_core::{widget_ids, widget, Positionable, Widget, Labelable, Ui, position::Place, Sizeable};
use conrod_glium::Renderer;
use glium::{glutin::{event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, ContextBuilder, event::WindowEvent}, Display, backend::glutin};
use crate::{game, managers::{render, scene, assets::get_full_asset_path}, convert_key};

widget_ids!(struct Ids { text });

pub fn start_game(debug_mode: DebugMode) {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new(); 
    let cb = ContextBuilder::new().with_srgb(false);
    let mut display = Display::new(wb, cb, &event_loop).unwrap();

    let mut frames_count: usize = 0;
    let mut now = std::time::Instant::now();

    let mut ui = conrod_core::UiBuilder::new([2560 as f64, 1080 as f64]).build();
    ui.fonts.insert_from_file(get_full_asset_path("fonts/JetBrainsMono-Regular.ttf")).expect("failed to add fonts");

    let ids = Ids::new(ui.widget_id_generator());

    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();
    let mut renderer = Renderer::new(&display).unwrap();

    game::start();
    scene::start(&mut display);

    let mut redraw_ui = true;
    let mut win_w = 0;
    let mut win_h = 0;

    event_loop.run(move |ev, _, control_flow| {
        game::update();
        scene::update();

        win_w = display.gl_window().window().inner_size().width;
        win_h = display.gl_window().window().inner_size().height;
        unsafe {
            render::ASPECT_RATIO = 
                win_w as f32 / win_h as f32;
        }

        match ev {
            glutin::glutin::event::Event::WindowEvent  { event, .. } => match event {
                WindowEvent::CursorMoved { device_id: _, position, modifiers: _ } => {
                    let tx = |x: conrod_core::Scalar| x - win_w as f64 / 2.0;
                    let ty = |y: conrod_core::Scalar| -(y - win_h as f64 / 2.0);

                    let n = Instant::now();
                    ui.handle_event(
                        conrod_core::event::Input::Motion(
                            conrod_core::input::Motion::MouseCursor { x: tx(position.x), y: ty(position.y) }));
                    println!("{:?}", n.elapsed());
                },
                WindowEvent::MouseInput { device_id: _, state, button, modifiers: _ } => {
                    match state {
                        glium::glutin::event::ElementState::Pressed => {
                            match button {
                                glium::glutin::event::MouseButton::Left => ui.handle_event(
                                    conrod_core::event::Input::Press(
                                        conrod_core::input::Button::Mouse(conrod_core::input::MouseButton::Left)).into()),
                                glium::glutin::event::MouseButton::Right => ui.handle_event(
                                    conrod_core::event::Input::Press(
                                        conrod_core::input::Button::Mouse(conrod_core::input::MouseButton::Right)).into()),
                                glium::glutin::event::MouseButton::Middle => ui.handle_event(
                                    conrod_core::event::Input::Press(
                                        conrod_core::input::Button::Mouse(conrod_core::input::MouseButton::Middle)).into()),
                                _ => (),
                            }
                        },

                        glium::glutin::event::ElementState::Released => { 
                            match button {
                                glium::glutin::event::MouseButton::Left => ui.handle_event(
                                    conrod_core::event::Input::Release(
                                        conrod_core::input::Button::Mouse(conrod_core::input::MouseButton::Left)).into()),
                                glium::glutin::event::MouseButton::Right => ui.handle_event(
                                    conrod_core::event::Input::Release(
                                        conrod_core::input::Button::Mouse(conrod_core::input::MouseButton::Right)).into()),
                                glium::glutin::event::MouseButton::Middle => ui.handle_event(
                                    conrod_core::event::Input::Release(
                                        conrod_core::input::Button::Mouse(conrod_core::input::MouseButton::Middle)).into()),
                                _ => (),
                            }
                        }
                    };
                    redraw_ui = true;
                },
                WindowEvent::KeyboardInput { device_id: _, input, is_synthetic: _ } => {
                    match input.virtual_keycode {
                        Some(code) => {
                            let key = convert_key!(code);
                            match input.state {
                                glium::glutin::event::ElementState::Pressed => ui.handle_event(
                                    conrod_core::event::Input::Press(conrod_core::input::Button::Keyboard(key)).into()),
                                glium::glutin::event::ElementState::Released => ui.handle_event(
                                    conrod_core::event::Input::Release(conrod_core::input::Button::Keyboard(key)).into()),
                            }
                        },
                        None => ()
                    }
                },
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::ExitWithCode(1);
                },
                WindowEvent::Resized(size) => {
                    ui.win_w = size.width as f64;
                    ui.win_h = size.height as f64;
                    redraw_ui = true;
                },
                _ => (),
            },
            _ => (),
        }


        let mut target = display.draw();

        render::draw(&mut target);
        game::render();
        scene::render(&mut display, &mut target);
        if redraw_ui == true {
            set_ui(&mut ui, &ids);
            redraw_ui = false;
        }
        
        let primitives = ui.draw();
        renderer.fill(&display, primitives, &image_map);
        renderer.draw(&display, &mut target, &image_map).unwrap();

        target.finish().unwrap();

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);

        match debug_mode {
            DebugMode::None => (),
            _ => {
                let fps = get_fps(&now, &frames_count);
                if fps.is_some() {
                    let fps = fps.unwrap();
                    display.gl_window().window().set_title(&format!("very cool window: {fps} fps"));
                    frames_count = 0;
                    now = Instant::now();
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
    Full
}

fn set_ui(ui: &mut Ui, ids: &Ids) {
    let mut ui_cell = ui.set_widgets();
    widget::Button::new()
        .middle_of(ui_cell.window)
        .y_dimension(conrod_core::position::Dimension::Absolute(100.0))
        .x_dimension(conrod_core::position::Dimension::Absolute(300.0))
        .label("test button")
        .label_font_size(32)
        .set(ids.text, &mut ui_cell);
}

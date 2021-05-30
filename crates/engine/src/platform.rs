use std::cmp::min;
use std::sync::{Arc, Mutex};

use sdl2::audio::AudioCallback as SdlAudioCallback;
use sdl2::audio::AudioSpecDesired as SdlAudioSpecDesired;
use sdl2::controller::Axis as SdlAxis;
use sdl2::controller::Button as SdlButton;
use sdl2::event::Event as SdlEvent;
use sdl2::event::WindowEvent as SdlWindowEvent;
use sdl2::mouse::MouseButton as SdlMouseButton;
use sdl2::pixels::Color as SdlColor;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::Texture as SdlTexture;
use sdl2::render::WindowCanvas as SdlCanvas;
use sdl2::video::FullscreenType as SdlFullscreenType;
use sdl2::AudioSubsystem as SdlAudioSubsystem;
use sdl2::EventPump as SdlEventPump;
use sdl2::TimerSubsystem as SdlTimerSubsystem;

use crate::canvas::Canvas;
use crate::event::{ControllerAxis, ControllerButton, Event, MouseButton};
use crate::image::Image;
use crate::palette::Rgb24;

pub trait AudioCallback {
    fn audio_callback(&mut self, out: &mut [f32]);
}

struct MySdlAudioCallback {
    audio_callback: Arc<Mutex<dyn AudioCallback + Send>>,
}

impl SdlAudioCallback for MySdlAudioCallback {
    type Channel = f32;

    #[inline]
    fn callback(&mut self, out: &mut [f32]) {
        let mut audio_callback = self.audio_callback.lock().unwrap();
        audio_callback.audio_callback(out);
    }
}

pub struct Platform {
    sdl_canvas: SdlCanvas,
    sdl_texture: SdlTexture,
    sdl_timer: SdlTimerSubsystem,
    sdl_event_pump: SdlEventPump,
    sdl_audio: SdlAudioSubsystem,
}

impl Platform {
    #[inline]
    pub fn new(title: &str, width: u32, height: u32, scale: u32) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let sdl_video = sdl_context.video().unwrap();
        let sdl_window = sdl_video
            .window(title, width * scale, height * scale)
            .position_centered()
            .resizable()
            .build()
            .unwrap();
        let mut sdl_canvas = sdl_window.into_canvas().build().unwrap();
        let sdl_texture = sdl_canvas
            .texture_creator()
            .create_texture_streaming(PixelFormatEnum::RGB24, width, height)
            .unwrap();
        let sdl_timer = sdl_context.timer().unwrap();
        let sdl_event_pump = sdl_context.event_pump().unwrap();
        let sdl_audio = sdl_context.audio().unwrap();

        sdl_canvas
            .window_mut()
            .set_minimum_size(width, height)
            .unwrap();

        Platform {
            sdl_timer: sdl_timer,
            sdl_canvas: sdl_canvas,
            sdl_texture: sdl_texture,
            sdl_event_pump: sdl_event_pump,
            sdl_audio: sdl_audio,
        }
    }

    #[inline]
    pub fn window_pos(&self) -> (i32, i32) {
        self.sdl_canvas.window().position()
    }

    #[inline]
    pub fn window_size(&self) -> (u32, u32) {
        self.sdl_canvas.window().size()
    }

    #[inline]
    pub fn window_title(&self) -> &str {
        self.sdl_canvas.window().title()
    }

    #[inline]
    pub fn set_window_title(&mut self, title: &str) {
        self.sdl_canvas.window_mut().set_title(title).unwrap();
    }

    #[inline]
    pub fn set_window_icon(&mut self, icon: &Image, scale: u32) {
        //
    }

    #[inline]
    pub fn is_fullscreen(&self) -> bool {
        self.sdl_canvas.window().fullscreen_state() == SdlFullscreenType::True
    }

    #[inline]
    pub fn set_fullscreen(&mut self, is_fullscreen: bool) {
        if is_fullscreen {
            self.sdl_canvas
                .window_mut()
                .set_fullscreen(SdlFullscreenType::True)
                .unwrap();
        } else {
            self.sdl_canvas
                .window_mut()
                .set_fullscreen(SdlFullscreenType::Off)
                .unwrap();
        }
    }

    #[inline]
    pub fn ticks(&self) -> u32 {
        self.sdl_timer.ticks()
    }

    #[inline]
    pub fn delay(&mut self, ms: u32) {
        self.sdl_timer.delay(ms);
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        loop {
            let sdl_event = self.sdl_event_pump.poll_event();

            if sdl_event.is_none() {
                return None;
            }

            let event = match sdl_event.unwrap() {
                //
                // System Events
                //
                SdlEvent::Quit { .. } => Event::Quit,

                SdlEvent::DropFile { filename, .. } => Event::DropFile { filename: filename },

                //
                // Window Events
                //
                SdlEvent::Window { win_event, .. } => match win_event {
                    /*
                    WindowShown,
                    WindowHidden,
                    */
                    SdlWindowEvent::Moved(x, y) => Event::WindowMoved { x: x, y: y },

                    SdlWindowEvent::Resized(width, height) => Event::WindowResized {
                        width: width,
                        height: height,
                    },

                    _ => continue,
                    /*
                    WindowMinimized,
                    WindowMaximized,
                    WindowEnter,
                    WindowLeave,
                    WindowFocusGained,
                    WindowFocusLost,
                    WindowClose,
                    */
                },

                //
                // Key Events
                //
                SdlEvent::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => Event::KeyDown {
                    key: scancode as u32,
                },

                SdlEvent::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => Event::KeyUp {
                    key: scancode as u32,
                },

                SdlEvent::TextInput { text, .. } => Event::TextInput { text: text },

                //
                // Mouse Events
                //
                SdlEvent::MouseMotion { x, y, .. } => Event::MouseMotion { x: x, y: y },

                SdlEvent::MouseButtonDown { mouse_btn, .. } => Event::MouseButtonDown {
                    button: match mouse_btn {
                        SdlMouseButton::Left => MouseButton::Left,
                        SdlMouseButton::Middle => MouseButton::Middle,
                        SdlMouseButton::Right => MouseButton::Right,
                        SdlMouseButton::X1 => MouseButton::X1,
                        SdlMouseButton::X2 => MouseButton::X2,
                        SdlMouseButton::Unknown => MouseButton::Unknown,
                    },
                },

                SdlEvent::MouseButtonUp { mouse_btn, .. } => Event::MouseButtonUp {
                    button: match mouse_btn {
                        SdlMouseButton::Left => MouseButton::Left,
                        SdlMouseButton::Middle => MouseButton::Middle,
                        SdlMouseButton::Right => MouseButton::Right,
                        SdlMouseButton::X1 => MouseButton::X1,
                        SdlMouseButton::X2 => MouseButton::X2,
                        SdlMouseButton::Unknown => MouseButton::Unknown,
                    },
                },

                SdlEvent::MouseWheel { x, y, .. } => Event::MouseWheel { x: x, y: y },

                //
                // Controller Events
                //
                SdlEvent::ControllerAxisMotion {
                    which, axis, value, ..
                } => Event::ControllerAxisMotion {
                    which: which,
                    axis: match axis {
                        SdlAxis::LeftX => ControllerAxis::LeftX,
                        SdlAxis::LeftY => ControllerAxis::LeftY,
                        SdlAxis::RightX => ControllerAxis::RightX,
                        SdlAxis::RightY => ControllerAxis::RightY,
                        SdlAxis::TriggerLeft => ControllerAxis::TriggerLeft,
                        SdlAxis::TriggerRight => ControllerAxis::TriggerRight,
                    },
                    value: value as i32,
                },

                SdlEvent::ControllerButtonDown { which, button, .. } => {
                    Event::ControllerButtonDown {
                        which: which,
                        button: match button {
                            SdlButton::A => ControllerButton::A,
                            SdlButton::B => ControllerButton::B,
                            SdlButton::X => ControllerButton::X,
                            SdlButton::Y => ControllerButton::Y,
                            SdlButton::Back => ControllerButton::Back,
                            SdlButton::Guide => ControllerButton::Guide,
                            SdlButton::Start => ControllerButton::Start,
                            SdlButton::LeftStick => ControllerButton::LeftStick,
                            SdlButton::RightStick => ControllerButton::RightStick,
                            SdlButton::LeftShoulder => ControllerButton::LeftShoulder,
                            SdlButton::RightShoulder => ControllerButton::RightShoulder,
                            SdlButton::DPadUp => ControllerButton::DPadUp,
                            SdlButton::DPadDown => ControllerButton::DPadDown,
                            SdlButton::DPadLeft => ControllerButton::DPadLeft,
                            SdlButton::DPadRight => ControllerButton::DPadRight,
                        },
                    }
                }

                SdlEvent::ControllerButtonUp { which, button, .. } => Event::ControllerButtonUp {
                    which: which,
                    button: match button {
                        SdlButton::A => ControllerButton::A,
                        SdlButton::B => ControllerButton::B,
                        SdlButton::X => ControllerButton::X,
                        SdlButton::Y => ControllerButton::Y,
                        SdlButton::Back => ControllerButton::Back,
                        SdlButton::Guide => ControllerButton::Guide,
                        SdlButton::Start => ControllerButton::Start,
                        SdlButton::LeftStick => ControllerButton::LeftStick,
                        SdlButton::RightStick => ControllerButton::RightStick,
                        SdlButton::LeftShoulder => ControllerButton::LeftShoulder,
                        SdlButton::RightShoulder => ControllerButton::RightShoulder,
                        SdlButton::DPadUp => ControllerButton::DPadUp,
                        SdlButton::DPadDown => ControllerButton::DPadDown,
                        SdlButton::DPadLeft => ControllerButton::DPadLeft,
                        SdlButton::DPadRight => ControllerButton::DPadRight,
                    },
                },

                //
                // Default
                //
                _ => continue,
            };

            return Some(event);
        }
    }

    pub fn render_screen(&mut self, screen: &Image, bg_color: Rgb24) {
        let screen_width = screen.width();
        let screen_height = screen.height();
        let screen_data = screen.data();
        let screen_palette = screen.palette();

        self.sdl_texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for i in 0..screen_height as usize {
                    for j in 0..screen_width as usize {
                        let offset = i * pitch + j * 3;
                        let color = screen_palette.display_color(screen_data[i][j]);

                        buffer[offset] = ((color >> 16) & 0xff) as u8;
                        buffer[offset + 1] = ((color >> 8) & 0xff) as u8;
                        buffer[offset + 2] = (color & 0xff) as u8;
                    }
                }
            })
            .unwrap();

        self.sdl_canvas.set_draw_color(SdlColor::RGB(
            ((bg_color >> 16) & 0xff) as u8,
            ((bg_color >> 8) & 0xff) as u8,
            (bg_color & 0xff) as u8,
        ));

        self.sdl_canvas.clear();

        let (window_width, window_height) = self.window_size();
        let screen_scale = min(window_width / screen_width, window_height / screen_height);
        let screen_x = (window_width - screen_width * screen_scale) / 2;
        let screen_y = (window_height - screen_height * screen_scale) / 2;

        let dst = SdlRect::new(
            screen_x as i32,
            screen_y as i32,
            screen_width * screen_scale,
            screen_height * screen_scale,
        );

        self.sdl_canvas
            .copy(&self.sdl_texture, None, Some(dst))
            .unwrap();

        self.sdl_canvas.present();
    }

    #[inline]
    pub fn init_audio(
        &mut self,
        sample_rate: u32,
        channels: u32,
        sample_count: u32,
        audio_callback: Arc<Mutex<dyn AudioCallback + Send>>,
    ) {
        let spec = SdlAudioSpecDesired {
            freq: Some(sample_rate as i32),
            channels: Some(channels as u8),
            samples: Some(sample_count as u16),
        };

        let device = self
            .sdl_audio
            .open_playback(None, &spec, |_| MySdlAudioCallback {
                audio_callback: audio_callback,
            })
            .unwrap();

        device.resume();
    }
}

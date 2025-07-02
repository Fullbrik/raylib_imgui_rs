mod frame_state;
mod maps;
mod clipboard;
pub mod image;

use std::ptr;
use raylib::prelude::*;
use imgui::{BackendFlags, ConfigFlags, DrawCmd, DrawIdx, DrawVert, Key, MouseCursor, TextureId, Ui};
use imgui::internal::{RawCast, RawWrapper};
use crate::clipboard::ClipboardBackend;
use crate::frame_state::FrameState;
use crate::maps::{KEYBOARD_MAP, MOUSE_CURSOR_MAP};

pub struct Renderer {
	current_cursor: Option<MouseCursor>,
	last_frame_state: FrameState,

	font_texture: Texture2D,
}

impl Renderer {
	/// Create a renderer
	pub fn create(imgui_context: &mut imgui::Context, raylib_handle: &mut RaylibHandle, raylib_thread: &RaylibThread) -> Self {
		KEYBOARD_MAP.len(); // Preload the keymap so we don't have to create it on the first frame

		Self::setup_context(imgui_context);

		let font_texture = Self::reload_fonts_impl(imgui_context, raylib_handle, raylib_thread);

		Self {
			current_cursor: Some(MouseCursor::Arrow),
			last_frame_state: FrameState::new(raylib_handle),

			font_texture,
		}
	}

	fn setup_context(imgui_context: &mut imgui::Context) {
		imgui_context.set_platform_name(Some("imgui_impl_raylib".to_string()));

		let io = imgui_context.io_mut();
		io.backend_flags.insert(BackendFlags::HAS_GAMEPAD | BackendFlags::HAS_SET_MOUSE_POS | BackendFlags::HAS_MOUSE_CURSORS);
		io.mouse_pos = [0.0, 0.0];

		imgui_context.set_clipboard_backend(ClipboardBackend);
	}

	/// Update the imgui context state. Call this before new_frame()
	pub fn update(&mut self, imgui_context: &mut imgui::Context, raylib_handle: &mut RaylibHandle) {
		self.update_display(imgui_context, raylib_handle);
		self.update_mouse(imgui_context, raylib_handle);
		self.process_events(imgui_context, raylib_handle);
	}

	fn update_display(&mut self, imgui_context: &mut imgui::Context, raylib_handle: &mut RaylibHandle) {
		let resolution_scale = raylib_handle.get_window_scale_dpi();

		let io = imgui_context.io_mut();

		io.display_size = [raylib_handle.get_screen_width() as _, raylib_handle.get_screen_height() as _];

		io.display_framebuffer_scale = [resolution_scale.x, resolution_scale.y];

		io.delta_time = raylib_handle.get_frame_time();
	}

	fn update_mouse(&mut self, imgui_context: &mut imgui::Context, raylib_handle: &mut RaylibHandle) {
		let io = imgui_context.io();
		if io.backend_flags.contains(BackendFlags::HAS_MOUSE_CURSORS) {
			if !io.config_flags.contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE) {
				let imgui_cursor = imgui_context.mouse_cursor();

				if self.current_cursor != imgui_cursor || io.mouse_draw_cursor {
					self.current_cursor = imgui_cursor;

					if io.mouse_draw_cursor || imgui_cursor.is_none() {
						raylib_handle.hide_cursor();
					} else {
						raylib_handle.show_cursor();

						if !io.config_flags.contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE) {
							if let Some(cursor) = imgui_cursor {
								raylib_handle.set_mouse_cursor(MOUSE_CURSOR_MAP[cursor as usize])
							} else {
								raylib_handle.set_mouse_cursor(consts::MouseCursor::MOUSE_CURSOR_DEFAULT);
							}
						}
					}
				}
			}
		}
	}

	fn process_events(&mut self, imgui_context: &mut imgui::Context, raylib_handle: &mut RaylibHandle) {
		let io = imgui_context.io_mut();

		let is_window_focused = raylib_handle.is_window_focused();
		if self.last_frame_state.is_window_focused != is_window_focused {
			unsafe {
				imgui::sys::ImGuiIO_AddFocusEvent(io.raw_mut(), is_window_focused);
			}

			self.last_frame_state.is_window_focused = is_window_focused;
		}

		let is_control_pressed = raylib_handle.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL) | raylib_handle.is_key_down(KeyboardKey::KEY_LEFT_CONTROL);
		if self.last_frame_state.is_control_pressed != is_control_pressed {
			io.add_key_event(Key::ModCtrl, is_control_pressed);
			self.last_frame_state.is_control_pressed = is_control_pressed;
		}

		let is_shift_pressed = raylib_handle.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) | raylib_handle.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
		if self.last_frame_state.is_shift_pressed != is_shift_pressed {
			io.add_key_event(Key::ModShift, is_shift_pressed);
			self.last_frame_state.is_shift_pressed = is_shift_pressed;
		}

		let is_alt_pressed = raylib_handle.is_key_down(KeyboardKey::KEY_RIGHT_ALT) | raylib_handle.is_key_down(KeyboardKey::KEY_LEFT_ALT);
		if self.last_frame_state.is_alt_pressed != is_alt_pressed {
			io.add_key_event(Key::ModAlt, is_alt_pressed);
			self.last_frame_state.is_alt_pressed = is_alt_pressed;
		}

		let is_super_pressed = raylib_handle.is_key_down(KeyboardKey::KEY_RIGHT_SUPER) | raylib_handle.is_key_down(KeyboardKey::KEY_LEFT_SUPER);
		if self.last_frame_state.is_super_pressed != is_super_pressed {
			io.add_key_event(Key::ModSuper, is_super_pressed);
			self.last_frame_state.is_super_pressed = is_super_pressed;
		}

		for (&rl_key, &imgui_key) in KEYBOARD_MAP.iter() {
			if raylib_handle.is_key_released(rl_key) {
				io.add_key_event(imgui_key, false);
			} else if raylib_handle.is_key_pressed(rl_key) {
				io.add_key_event(imgui_key, true);
			}
		}

		if io.want_capture_keyboard {
			while let Some(pressed) = raylib_handle.get_char_pressed() {
				io.add_input_character(pressed);
			}
		}

		if!io.want_set_mouse_pos {
			io.add_mouse_pos_event([raylib_handle.get_mouse_x() as _, raylib_handle.get_mouse_y() as _]);
		}

		let mut set_mouse_event = |rl_mouse, imgui_mouse| {
			if raylib_handle.is_mouse_button_pressed(rl_mouse) {
				io.add_mouse_button_event(imgui_mouse, true);
			} else if raylib_handle.is_mouse_button_released(rl_mouse) {
				io.add_mouse_button_event(imgui_mouse, false);
			}
		};

		set_mouse_event(MouseButton::MOUSE_BUTTON_LEFT, imgui::MouseButton::Left);
		set_mouse_event(MouseButton::MOUSE_BUTTON_RIGHT, imgui::MouseButton::Right);
		set_mouse_event(MouseButton::MOUSE_BUTTON_MIDDLE, imgui::MouseButton::Middle);
		set_mouse_event(MouseButton::MOUSE_BUTTON_FORWARD, imgui::MouseButton::Extra1);
		set_mouse_event(MouseButton::MOUSE_BUTTON_BACK, imgui::MouseButton::Extra2);

		let mouse_wheel = raylib_handle.get_mouse_wheel_move_v();
		io.add_mouse_wheel_event([mouse_wheel.x, mouse_wheel.y]);

		if io.config_flags.contains(ConfigFlags::NAV_ENABLE_GAMEPAD) && raylib_handle.is_gamepad_available(0) {
			let mut handle_gamepad_button_event = |rl_button, imgui_button| {
				if raylib_handle.is_gamepad_button_pressed(0, rl_button) {
					io.add_key_event(imgui_button, true);
				} else if raylib_handle.is_gamepad_button_released(0, rl_button) {
					io.add_key_event(imgui_button, false);
				}
			};

			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP, Key::GamepadDpadUp);
			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT, Key::GamepadDpadRight);
			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN, Key::GamepadDpadDown);
			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT, Key::GamepadDpadLeft);

			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_UP, Key::GamepadFaceUp);
			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT, Key::GamepadFaceLeft);
			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN, Key::GamepadFaceDown);
			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_LEFT, Key::GamepadFaceRight);

			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_1, Key::GamepadL1);
			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_2, Key::GamepadL2);
			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1, Key::GamepadR1);
			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_2, Key::GamepadR2);
			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_LEFT_THUMB, Key::GamepadL3);
			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_RIGHT_THUMB, Key::GamepadR3);

			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_MIDDLE_LEFT, Key::GamepadStart);
			handle_gamepad_button_event(GamepadButton::GAMEPAD_BUTTON_MIDDLE_RIGHT, Key::GamepadBack);

			let mut handle_gamepad_stick_event = |axis, neg_key, pos_key| {
				const DEAD_ZONE: f32 = 0.2;

				let axis_value = raylib_handle.get_gamepad_axis_movement(0, axis);

				io.add_key_analog_event(neg_key, axis_value < -DEAD_ZONE, if axis_value < -DEAD_ZONE { -axis_value } else { 0.0 });
				io.add_key_analog_event(pos_key, axis_value > DEAD_ZONE, if axis_value > DEAD_ZONE { axis_value } else { 0.0 });
			};

			// left stick
			handle_gamepad_stick_event(GamepadAxis::GAMEPAD_AXIS_LEFT_X, Key::GamepadLStickLeft, Key::GamepadLStickRight);
			handle_gamepad_stick_event(GamepadAxis::GAMEPAD_AXIS_LEFT_Y, Key::GamepadLStickUp, Key::GamepadLStickDown);

			// right stick
			handle_gamepad_stick_event(GamepadAxis::GAMEPAD_AXIS_RIGHT_X, Key::GamepadRStickLeft, Key::GamepadRStickRight);
			handle_gamepad_stick_event(GamepadAxis::GAMEPAD_AXIS_RIGHT_Y, Key::GamepadRStickUp, Key::GamepadRStickDown);
		}
	}

	/// Render the frame. Call this after drawing all your imgui stuff.
	pub fn render(&self, imgui_context: &mut imgui::Context, draw: &mut RaylibDrawHandle) {
		let io = imgui_context.io();
		
		let display_framebuffer_scale = if draw.get_window_state().window_highdpi() {
			io.display_framebuffer_scale
		} else { 
			[1.0, 1.0]
		};
		
		let display_size = io.display_size;
		let draw_data = imgui_context.render();

		let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
		let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];

		if !(fb_width > 0.0 && fb_height > 0.0) || draw_data.draw_lists_count() == 0 {
			return;
		}

		unsafe {
			ffi::rlDrawRenderBatchActive();
			ffi::rlDisableBackfaceCulling();
		}

		for draw_list in draw_data.draw_lists() {
			for command in draw_list.commands() {
				match command {
					DrawCmd::Elements { count, cmd_params } => {
						unsafe {
							Self::enable_scissor(
								cmd_params.clip_rect[0] - draw_data.display_pos[0],
								cmd_params.clip_rect[1] - draw_data.display_pos[1],
								cmd_params.clip_rect[2] - (cmd_params.clip_rect[0] - draw_data.display_pos[0]),
								cmd_params.clip_rect[3] - (cmd_params.clip_rect[1] - draw_data.display_pos[1]),
								display_framebuffer_scale,
								display_size,
							);

							Self::render_triangles(count, cmd_params.idx_offset, cmd_params.vtx_offset, draw_list.idx_buffer(), draw_list.vtx_buffer(), cmd_params.texture_id);

							ffi::rlDrawRenderBatchActive();
						}
					}
					DrawCmd::ResetRenderState => {
						// TODO: Figure out what to do here
						unsafe {
							ffi::rlSetTexture(0);
						}
					}
					DrawCmd::RawCallback { callback, raw_cmd } => {
						unsafe {
							callback(draw_list.raw(), raw_cmd);
						}
					}
				}
			}
		}

		unsafe {
			ffi::rlSetTexture(0);
			ffi::rlDisableScissorTest();
			ffi::rlEnableBackfaceCulling();
		}
	}

	unsafe fn enable_scissor(x: f32, y: f32, w: f32, h: f32, display_framebuffer_scale: [f32; 2], display_size: [f32; 2]) {
		ffi::rlEnableScissorTest();

		ffi::rlScissor(
			(x * display_framebuffer_scale[0]) as _,
			(display_size[1] - ((y + h).floor() * display_framebuffer_scale[1])) as _,
			(w * display_framebuffer_scale[0]) as _,
			(h * display_framebuffer_scale[1]) as _,
		);
	}

	unsafe fn render_triangles(count: usize, indx_start: usize, vtx_start: usize, indx_buffer: &[DrawIdx], vert_buffer: &[DrawVert], texture_id: TextureId) {
		if count < 3 { return; }

		ffi::rlBegin(ffi::RL_TRIANGLES as _);
		ffi::rlSetTexture(texture_id.id() as _);

		for i in 0..count {
			let indx = indx_buffer[indx_start + i] as usize;
			Self::draw_vertex(vert_buffer[vtx_start + indx]);
		}

		ffi::rlEnd();
	}

	unsafe fn draw_vertex(vert: DrawVert) {
		ffi::rlColor4ub(vert.col[0], vert.col[1], vert.col[2], vert.col[3]);
		ffi::rlTexCoord2f(vert.uv[0], vert.uv[1]);
		ffi::rlVertex2f(vert.pos[0], vert.pos[1]);
	}

	pub fn reload_fonts(&mut self, imgui_context: &mut imgui::Context, raylib_handle: &mut RaylibHandle, raylib_thread: &RaylibThread) {
		self.font_texture = Self::reload_fonts_impl(imgui_context, raylib_handle, raylib_thread);
	}

	fn reload_fonts_impl(imgui_context: &mut imgui::Context, raylib_handle: &mut RaylibHandle, raylib_thread: &RaylibThread) -> Texture2D {
		let atlas = imgui_context.fonts().build_rgba32_texture();
		let image = Image::gen_image_color(atlas.width as _, atlas.height as _, Color::WHITE);
		
		unsafe {
			ptr::copy(atlas.data.as_ptr(), image.data() as _, atlas.width as usize * atlas.height as usize * 4);

			let font_texture = raylib_handle.load_texture_from_image(raylib_thread, &image).unwrap(); // TODO: Don't unwrap
			drop(image);

			imgui_context.fonts().tex_id = TextureId::from(font_texture.id as usize);

			font_texture
		}
	}
}
use imgui::{Context, FontSource};
use raylib::prelude::*;
use raylib_imgui_rs::{Renderer};
use raylib_imgui_rs::image::ImageExt;

fn main() {
	let (mut rl, thread) = raylib::init()
		.size(640, 480)
		.title("Raylib texture example")
		.resizable()
		.build();

	let mut imgui = Context::create();
	imgui.fonts().add_font(&[FontSource::DefaultFontData { config: None }]);

	let mut renderer = Renderer::create(&mut imgui, &mut rl, &thread);

	// Load the texture we are going to use in imgui
	let image = Image::load_image_from_mem(".png", include_bytes!("assets/TestImage.png")).unwrap();
	let texture = rl.load_texture_from_image(&thread, &image).unwrap();


	while !rl.window_should_close() {
		renderer.update(&mut imgui, &mut rl);

		{
			let ui = imgui.new_frame();

			if let Some(_token) = ui.window("Texture").begin() {
				ui.image_scaled(&texture, 128, 128);
				ui.image(&texture);
			}
		}

		{
			let mut d = rl.begin_drawing(&thread);

			d.clear_background(Color::WHITE);
			d.draw_fps(12, 12);

			renderer.render(&mut imgui, &mut d);
		}
	}
}
use imgui::{Context, FontSource};
use raylib::prelude::*;
use raylib_imgui_rs::Renderer;

fn main() {
	let (mut rl, thread) = raylib::init()
		.size(640, 480)
		.title("Demo window example")
		.resizable()
		.build();

	let mut imgui = Context::create();
	imgui.fonts().add_font(&[FontSource::DefaultFontData { config: None }]);

	let mut renderer = Renderer::create(&mut imgui, &mut rl, &thread);

	while !rl.window_should_close() {
		renderer.update(&mut imgui, &mut rl);

		{
			let ui = imgui.new_frame();
			ui.show_demo_window(&mut true);
		}

		{
			let mut d = rl.begin_drawing(&thread);

			d.clear_background(Color::WHITE);
			d.draw_fps(12, 12);

			renderer.render(&mut imgui, &mut d);
		}
	}
}
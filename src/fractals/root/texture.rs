//! # Complex sequences.
//! src/fractals/root/texture.rs
//!
//! Draw the texture for a Newton fractal.

use std::{cell, error, rc, time};

use glium::{self, backend};
use imgui;
use imgui_glium_renderer;
use complex;

use crate::structures::{configuration, color, computations};
use crate::fractals;
use crate::support::rendering;


/// # `Root`, drawing board for `imgui`.
pub struct Root<F, D> 
where
	F: Fn(complex::Algebraic) -> complex::Algebraic,
	D: Fn(complex::Algebraic) -> complex::Algebraic,
{
	function: F,
	derivative: D,
	texture_id: Option<imgui::TextureId>,
	
	iterations_total: usize,
	generation_time: Option<time::Duration>,

	// Parameters.
	pub size: [u32; 2],
	/// Size: [width, height].
	pub information_size: [f32; 2],
	pub scale: complex::Real,
	pub position: [complex::Real; 2],
	pub zoom: complex::Real,
	pub iterations: usize,
	pub threshold: complex::Real,
	pub method_id: usize,
	pub degree0: f64,

	// Variables to check if state is modified.
	zoom_last: complex::Real,
	position_last: [complex::Real; 2],
	iterations_last: usize,
	threshold_last: complex::Real,
	method_id_last: usize,
	scale_last: complex::Real,
	degree0_last: f64,

	/// Graphics.
	color_no_root: color::Rgb,

}

impl<F, D> Root<F, D> 
where
	F: Fn(complex::Algebraic) -> complex::Algebraic,
	D: Fn(complex::Algebraic) -> complex::Algebraic,
{
	/// Instantiate and returns a link to a new `Root`.
	pub fn new(
		function: F,
		derivative: D,
		information_size: [f32; 2],
		position: [complex::Real; 2], 
		scale: complex::Real,
		zoom: complex::Real,
		iterations: usize,
		threshold: complex::Real,
		method_id: usize,
		color_no_root: color::Rgb,
	) -> rc::Rc<cell::RefCell<Root<F, D>>> {
		rc::Rc::new(cell::RefCell::new(Root {
			function,
			derivative,
			iterations_total: 0usize,
			texture_id: Option::None,
			size: [0, 0],
			information_size, 
			scale,
			generation_time: Option::None,
			position,
			zoom,
			iterations,
			threshold,
			method_id,
			degree0: 0.0,

			zoom_last: 1.0,
			position_last: [0.0, 0.0],
			iterations_last: 0,
			threshold_last: 0.0,
			method_id_last: 0,
			scale_last: 1.0,
			degree0_last: 0.0,

			color_no_root,
		}))
	}

	/// Check the fields of the `Root` versus their `last` counterpart.
	/// 
	/// Returns `true` if any of them is different.
	pub fn is_state_updated(self: &mut Self) -> bool {
		let mut updated: bool = true;

		if self.zoom_last != self.zoom {
			self.zoom_last = self.zoom;
		} else if self.position_last != self.position {
			self.position_last = self.position;
		} else if self.iterations_last != self.iterations {
			self.iterations_last = self.iterations;
		} else if self.threshold_last != self.threshold {
			self.threshold_last = self.threshold;
		} else if self.method_id_last != self.method_id {
			self.method_id_last = self.method_id;
		} else if self.scale_last != self.scale {
			self.scale_last = self.scale;
		} else if self.degree0_last != self.degree0 {
			self.degree0_last = self.degree0;
		} else {	
			updated = false;
		}

		updated
	}
}

impl<F, D> fractals::textures::Fractal for Root<F, D> 
where
	F: Fn(complex::Algebraic) -> complex::Algebraic,
	D: Fn(complex::Algebraic) -> complex::Algebraic,
{
	fn update_size(self: &mut Self, new_size: [u32; 2]) -> () {
		self.size = new_size
	}
	
	fn register_texture<Facade>(
        &mut self,
        gl_context: &Facade,
		global_settings: rc::Rc<cell::RefCell<configuration::GlobalSettings>>,
        textures: &mut imgui::Textures<imgui_glium_renderer::Texture>,
    ) -> Result<(), Box<dyn error::Error>>
    where
        Facade: backend::Facade,
    {	
		let scale: complex::Real = global_settings.borrow().resolution_scale;
		self.scale = scale;
		let scaled_size: [usize; 2] = [
			(self.size[0] as complex::Real / scale) as usize, 
			(self.size[1] as complex::Real / scale) as usize,
		];
		let mut root_finder: fractals::root::maths::RootFinder<&F, &D> = fractals::root::maths::RootFinder::new(
			&self.function, 
			&self.derivative, 
			self.threshold,
			self.iterations, 
			scaled_size, 
			self.position, 
			self.zoom / scale,
		);
			
		// Texture generation.
		let generation_start: time::Instant = time::Instant::now();

		let data: computations::Data = match self.method_id {
			0 => {
				let table: Vec<Vec<computations::IsRoot>> = root_finder.limit_on_screen_newton();
				let mut newton_converter: fractals::tables::NewtonConverter;
				newton_converter = fractals::tables::NewtonConverter::new(
					root_finder.get_roots(),
					root_finder.get_threshold(),
					self.color_no_root,
					self.iterations,
					global_settings.borrow().color_mode,
				);

				newton_converter.convert(table)
			},
			_ => {
				let table: Vec<Vec<complex::Polar>> = root_finder.limit_on_screen_position();
				let mut position_converter: fractals::tables::PositionConverter;
				position_converter = fractals::tables::PositionConverter::new(self.degree0);

				position_converter.convert(table)
			},
		};

		self.iterations_total = data.iterations_total;
		self.generation_time = Option::Some(generation_start.elapsed());
		
		let render_result = rendering::render_texture(
			self.texture_id, 
			data.raw_pixels, 
			scaled_size, 
			gl_context, 
			textures,
			rendering::ColorFormat::RGB,
		);
		if let Result::Err(error) = &render_result {
			eprintln!("(!) fractals::root::texture::Root::register_texture()
render_texture error {}.", error);
		}

		self.texture_id = render_result.ok();

		let root_count: usize = root_finder.get_roots().len();
		eprintln!(
			"* Root {}: t={} zoom={} pos=({}; {}) roots={}", 
			self.method_id,
			&match self.generation_time {
				Option::None => String::from("()"),
				Option::Some(elapsed) => format!("{:?}", elapsed),
			}, 
			self.zoom,
			self.position[0],
			self.position[1],
			root_count,
		);

		if root_count > 23 {
			eprintln!("(!) Root: `root_count` high.")
		}

		Ok(())
	}

	/// Display the root fractal render and rendering information.
	fn show_textures(&self, ui: &imgui::Ui, information_position: [f32; 2]) {
        let draw_list_background: imgui::DrawListMut<'_> = ui.get_background_draw_list();

		// Render `Image` in the draw list.
		if let Some(texture_id) = self.texture_id {
			draw_list_background
				.add_image(texture_id, [0.0, 0.0], [self.size[0] as f32, self.size[1] as f32])
				.build();
		}
		
		ui.window(format!("Rendering: Fractal 'Root' (method {}). ", self.method_id))
            .size(self.information_size, imgui::Condition::FirstUseEver)
			.position(information_position, imgui::Condition::FirstUseEver)
            .build(|| {
				if let Some(generation_time) = self.generation_time 
					&& generation_time.as_millis() != 0
				{
					ui.text(format!("Size: ({}; {});
Pixels = {:.0}; 
Iterations = {};
Time = {:?}ms;
Speed = {} iterations/ ms;", 
						self.size[0],
						self.size[1],
						(self.size[0] * self.size[1]) as complex::Real / self.scale,
						self.iterations_total,
						generation_time.as_millis(),
						self.iterations_total as u128 / generation_time.as_millis(),
					));
				} else {
					ui.text(format!("(!) Error: no data."));
				}
			
			});
	}
}

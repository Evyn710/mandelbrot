use iced::widget::{canvas, container};
use iced::{mouse, Color, Element, Fill, Point, Rectangle, Renderer, Size, Theme};

use num::complex::Complex;

use std::time::{Duration, Instant};

struct Pixel {
    x: f32,
    y: f32,
    color: Color,
}

#[derive(Debug)]
enum Message {
    NoOp,
}

#[derive(Default)]
struct Mandelbrot {}

impl Mandelbrot {
    fn view(&self) -> Element<Message> {
        canvas(MandelbrotDrawing { pixels: Vec::new() })
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn update(&mut self, message: Message) {}
}

#[derive(Default)]
struct MandelbrotDrawing {
    pixels: Vec<Pixel>,
}

fn threaded_fractal_calc(start_column: i32, end_column: i32, bounds: &Rectangle) -> Vec<Pixel> {
    let mut result: Vec<Pixel> = Vec::new();
    for x in start_column..end_column as i32 {
        for y in 0..bounds.height as i32 {
            let i = (x as f32 - bounds.width * 2.0 / 3.0) / (bounds.width / 3.0);
            let j = (y as f32 - bounds.height / 2.0) / (bounds.width / 3.0);
            let c = Complex::new(i, j);
            let mut z = Complex::new(0.0, 0.0);
            let mut diverged = false;
            for _ in 0..100 {
                z = z * z + c;
                if !z.is_finite() {
                    diverged = true;
                    break;
                }
            }

            if !diverged {
                result.push(Pixel {
                    x: x as f32,
                    y: y as f32,
                    color: Color::BLACK,
                });
            }
        }
    }

    result
}

impl<Message> canvas::Program<Message> for MandelbrotDrawing {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        println!("{:#?}", bounds.size());
        let start = Instant::now();
        let background = canvas::Path::rectangle(Point::new(0.0, 0.0), bounds.size());
        frame.fill(&background, Color::WHITE);
        let result = threaded_fractal_calc(0, bounds.width as i32, &bounds);
        for pixel in result {
            let pixel_rectangle =
                canvas::Path::rectangle(Point::new(pixel.x, pixel.y), Size::new(1.0, 1.0));
            frame.fill(&pixel_rectangle, pixel.color);
        }
        let duration = start.elapsed();
        println!("{:#?}", duration);

        vec![frame.into_geometry()]
    }
}

fn main() -> iced::Result {
    iced::run("Mandelbrot", Mandelbrot::update, Mandelbrot::view)
}

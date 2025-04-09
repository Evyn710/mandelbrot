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
        for x in 0..bounds.size().width as i32 {
            for y in 0..bounds.size().height as i32 {
                let rectangle =
                    canvas::Path::rectangle(Point::new(x as f32, y as f32), Size::new(1.0, 1.0));
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
                    frame.fill(&rectangle, Color::BLACK);
                }
            }
        }
        let duration = start.elapsed();
        println!("{:#?}", duration);

        vec![frame.into_geometry()]
    }
}

fn main() -> iced::Result {
    iced::run("Mandelbrot", Mandelbrot::update, Mandelbrot::view)
}

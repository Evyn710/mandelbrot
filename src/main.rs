use iced::widget::canvas;
use iced::{mouse, Color, Element, Fill, Point, Rectangle, Renderer, Size, Theme};

use num::complex::Complex;

use std::sync::mpsc::channel;
use std::time::Instant;

use threadpool::ThreadPool;

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
        println!("Hello");
        canvas(MandelbrotDrawing {
            threadpool: ThreadPool::new(12),
        })
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn update(&mut self, message: Message) {}
}

#[derive(Default)]
struct MandelbrotDrawing {
    threadpool: ThreadPool,
}

fn threaded_fractal_calc(pool: &ThreadPool, bounds: Rectangle) -> Vec<Pixel> {
    let mut overall_result: Vec<Pixel> = Vec::new();

    let n_threads = pool.max_count();
    let n_jobs = 40;

    let pixel_thread_width = bounds.width / n_threads as f32;

    let (tx, rx) = channel();
    for i in 0..n_jobs {
        let tx = tx.clone();
        let start_column = i * pixel_thread_width as usize;
        let end_column = start_column + pixel_thread_width as usize;
        pool.execute(move || {
            let mut result: Vec<Pixel> = Vec::new();
            for x in start_column..end_column {
                for y in 0..bounds.height as usize {
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
            tx.send(result)
                .expect("channel will be there waiting for the result");
        });
    }

    for _ in 0..n_jobs {
        overall_result.append(&mut rx.recv().unwrap());
    }

    overall_result
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
        let result = threaded_fractal_calc(&self.threadpool, bounds);
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

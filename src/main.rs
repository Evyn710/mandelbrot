use iced::event::{self, Event};
use iced::widget::canvas;
use iced::{
    mouse, window, Color, Element, Fill, Point, Rectangle, Renderer, Size, Subscription, Theme,
};

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
    EventOccurred(Event),
}

#[derive(Debug)]
struct Mandelbrot {
    zoom_level: f32,
    current_mouse_location: Point,
    center_location: Point,
    window_size: Size,
}

impl Default for Mandelbrot {
    fn default() -> Self {
        Mandelbrot {
            zoom_level: 1.0,
            current_mouse_location: Point::new(-0.5, 0.0),
            center_location: Point::new(-0.5, 0.0),
            window_size: Size::new(1200.0, 720.0),
        }
    }
}

impl Mandelbrot {
    fn view(&self) -> Element<Message> {
        println!("{:#?}", self.center_location);
        canvas(MandelbrotDrawing {
            threadpool: ThreadPool::new(12),
            scale: self.zoom_level,
            center: self.center_location,
        })
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::EventOccurred(event) => {
                if let Event::Window(window::Event::Resized(size)) = event {
                    self.window_size = size;
                }
                if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
                    match delta {
                        mouse::ScrollDelta::Lines { x: _, y } => {
                            self.zoom_level = f32::max(1.0, self.zoom_level + y * 0.1);
                            self.center_location = Point {
                                x: self.current_mouse_location.x / self.window_size.width - 0.5,
                                y: -(self.current_mouse_location.y / self.window_size.height - 0.5),
                            };
                        }
                        mouse::ScrollDelta::Pixels { x: _, y: _ } => {}
                    }
                }
                if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
                    self.current_mouse_location = position;
                }
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }
}

#[derive(Default)]
struct MandelbrotDrawing {
    threadpool: ThreadPool,
    scale: f32,
    center: Point,
}

fn threaded_fractal_calc(
    pool: &ThreadPool,
    bounds: Rectangle,
    scale: f32,
    center: Point,
) -> Vec<Pixel> {
    let mut overall_result: Vec<Pixel> = Vec::new();

    let n_threads = pool.max_count();
    let n_jobs = 40;

    let pixel_thread_width = bounds.width / n_threads as f32;

    let (tx, rx) = channel();
    for i in 0..n_jobs {
        let tx = tx.clone();
        let start_row = i * pixel_thread_width as usize;
        let end_row = start_row + pixel_thread_width as usize;
        pool.execute(move || {
            let mut result: Vec<Pixel> = Vec::new();
            for x in 0..bounds.width as usize {
                for y in start_row..end_row {
                    let i = (x as f32 - bounds.width * 2.0 / 3.0) / (bounds.width / 3.0 * scale);
                    let j = (y as f32 - bounds.height / 2.0) / (bounds.width / 3.0 * scale);
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
        let start = Instant::now();
        let background = canvas::Path::rectangle(Point::new(0.0, 0.0), bounds.size());
        frame.fill(&background, Color::WHITE);
        let result = threaded_fractal_calc(&self.threadpool, bounds, self.scale, self.center);
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
    iced::application("Mandelbrot", Mandelbrot::update, Mandelbrot::view)
        .subscription(Mandelbrot::subscription)
        .run()
}

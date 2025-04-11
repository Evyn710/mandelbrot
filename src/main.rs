use bytes::Bytes;

use iced::event::{self, Event};
use iced::widget::image;
use iced::{
    mouse, window, Color, Element, Fill, Point, Rectangle, Renderer, Size, Subscription, Theme,
};

use num::complex::Complex;

use std::sync::mpsc::channel;
use std::time::Instant;

use threadpool::ThreadPool;

#[derive(Clone, Debug)]
struct Pixel {
    x: usize,
    y: usize,
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
    threadpool: ThreadPool,
    image: image::Handle,
}

impl Default for Mandelbrot {
    fn default() -> Self {
        Mandelbrot {
            zoom_level: 1.0,
            current_mouse_location: Point::new(-0.5, 0.0),
            center_location: Point::new(-0.5, 0.0),
            window_size: Size::new(1200.0, 720.0),
            threadpool: ThreadPool::new(8),
            image: image::Handle::from_rgba(0, 0, Vec::new()),
        }
    }
}

impl Mandelbrot {
    fn view(&self) -> Element<Message> {
        image(self.image.clone()).width(Fill).height(Fill).into()
    }

    fn update(&mut self, message: Message) {
        let mut should_draw = false;
        match message {
            Message::EventOccurred(event) => {
                if let Event::Window(window::Event::Resized(size)) = event {
                    self.window_size = size;
                    println!("x: {} y: {}", size.width as usize, size.height as usize);
                    should_draw = true;
                }
                if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
                    match delta {
                        mouse::ScrollDelta::Lines { x: _, y } => {
                            self.zoom_level = f32::max(1.0, self.zoom_level + y * 0.5);

                            should_draw = true;
                        }
                        mouse::ScrollDelta::Pixels { x: _, y: _ } => {}
                    }
                }
                if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
                    self.current_mouse_location = position;
                }
                if let Event::Mouse(mouse::Event::ButtonPressed(button)) = event {
                    if button == iced::mouse::Button::Left {
                        self.center_location = Point {
                            x: self.current_mouse_location.x / self.window_size.width - 1.0,
                            y: self.current_mouse_location.y / self.window_size.height - 0.5,
                        };

                        should_draw = true;
                    }
                }
            }
        }

        if should_draw {
            let start = Instant::now();
            self.image = threaded_fractal_calc(
                &self.threadpool,
                self.window_size,
                self.zoom_level,
                self.center_location,
            );
            println!("duration to calculate {:#?}", start.elapsed());
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }
}

fn threaded_fractal_calc(
    pool: &ThreadPool,
    bounds: Size,
    scale: f32,
    center: Point,
) -> image::Handle {
    let mut overall_result = Vec::with_capacity(bounds.width as usize);
    for _ in 0..bounds.width as usize {
        let mut column = Vec::with_capacity(bounds.height as usize);
        for _ in 0..bounds.height as usize {
            column.push(Color::TRANSPARENT);
        }
        overall_result.push(column);
    }

    let n_jobs = 32;

    let pixel_job_height = bounds.height / n_jobs as f32;

    let (tx, rx) = channel();
    for i in 0..n_jobs {
        let tx = tx.clone();
        let start_row = i * pixel_job_height as usize;
        let end_row = start_row + pixel_job_height as usize;
        pool.execute(move || {
            let mut result: Vec<Pixel> = Vec::new();
            let x_res = 3.0 / scale / bounds.width;
            let y_res = 2.0 / scale / bounds.height;
            for x in 0..bounds.width as usize {
                for y in start_row..end_row {
                    let i = center.x - x_res * bounds.width / 2.0 + x as f32 * x_res;
                    let j = center.y - y_res * bounds.height / 2.0 + y as f32 * y_res;
                    let c = Complex::new(i, j);
                    let mut z = Complex::new(0.0, 0.0);
                    let mut diverged = false;
                    for _ in 0..50 {
                        z = z * z + c;
                        if z.norm() >= 2.0 {
                            diverged = true;
                            break;
                        }
                    }

                    let mut color = Color::BLACK;
                    if diverged {
                        color = Color::WHITE;
                    }

                    result.push(Pixel { x, y, color });
                }
            }
            tx.send(result)
                .expect("channel will be there waiting for the result");
        });
    }

    for _ in 0..n_jobs {
        let pixels = rx.recv().unwrap();
        for pixel in pixels {
            overall_result[pixel.x][pixel.y] = pixel.color;
        }
    }

    let mut bytes: Vec<u8> =
        Vec::with_capacity(bounds.width as usize * bounds.height as usize * 4 as usize);
    for j in 0..bounds.height as usize {
        for i in 0..bounds.width as usize {
            if overall_result[i][j] == Color::BLACK {
                bytes.push(0);
                bytes.push(0);
                bytes.push(0);
                bytes.push(255);
            }
            if overall_result[i][j] == Color::WHITE {
                bytes.push(255);
                bytes.push(255);
                bytes.push(255);
                bytes.push(255);
            }
        }
    }

    image::Handle::from_rgba(
        bounds.width as u32,
        bounds.height as u32,
        Bytes::from(bytes),
    )
}

fn main() -> iced::Result {
    iced::application("Mandelbrot", Mandelbrot::update, Mandelbrot::view)
        .subscription(Mandelbrot::subscription)
        .run()
}

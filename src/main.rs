use bytes::Bytes;

use iced::event::{self, Event};
use iced::widget::{canvas, container, image, stack};
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
    current_mouse_location: Point,
    draw_bounding_box: bool,
    start_location: Point,
    end_location: Point,
    region: Rectangle,
    window_size: Size,
    threadpool: ThreadPool,
    image: image::Handle,
}

impl Default for Mandelbrot {
    fn default() -> Self {
        Mandelbrot {
            current_mouse_location: Point::new(-0.5, 0.0),
            draw_bounding_box: false,
            start_location: Point::default(),
            end_location: Point::default(),
            region: Rectangle::default(),
            window_size: Size::new(1200.0, 720.0),
            threadpool: ThreadPool::new(8),
            image: image::Handle::from_rgba(0, 0, Vec::new()),
        }
    }
}

impl Mandelbrot {
    fn view(&self) -> Element<Message> {
        stack![
            image(self.image.clone()),
            container(
                canvas(RectangleProgram {
                    region: Rectangle {
                        x: self.start_location.x,
                        y: self.start_location.y,
                        width: self.end_location.x - self.start_location.x,
                        height: self.end_location.y - self.start_location.y,
                    },
                    draw_bounding_box: self.draw_bounding_box
                })
                .width(Fill)
                .height(Fill),
            ),
        ]
        .into()
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
                if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
                    self.current_mouse_location = position;
                    self.end_location = position;
                }
                if let Event::Mouse(mouse::Event::ButtonPressed(button)) = event {
                    if button == iced::mouse::Button::Left {
                        self.start_location = Point {
                            x: self.current_mouse_location.x,
                            y: self.current_mouse_location.y,
                        };
                        self.draw_bounding_box = true;
                    }
                    if button == iced::mouse::Button::Right {
                        self.draw_bounding_box = false;
                    }
                }
                if let Event::Mouse(mouse::Event::ButtonReleased(button)) = event {
                    if button == iced::mouse::Button::Left {
                        if self.draw_bounding_box {
                            self.region = Rectangle {
                                x: self.start_location.x,
                                y: self.start_location.y,
                                width: self.end_location.x - self.start_location.x,
                                height: self.end_location.y - self.start_location.y,
                            };
                            should_draw = true;
                            self.draw_bounding_box = false;
                        }
                    }
                }
            }
        }

        if should_draw {
            let start = Instant::now();
            self.image = threaded_fractal_calc(&self.threadpool, self.window_size, self.region);
            println!("duration to calculate {:#?}", start.elapsed());
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }
}

fn threaded_fractal_calc(pool: &ThreadPool, bounds: Size, region: Rectangle) -> image::Handle {
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
            let x_res = 3.0 / bounds.width;
            let y_res = 2.0 / bounds.height;
            for x in 0..bounds.width as usize {
                for y in start_row..end_row {
                    let i = -0.5 - x_res * bounds.width / 2.0 + x as f32 * x_res;
                    let j = 0.0 - y_res * bounds.height / 2.0 + y as f32 * y_res;
                    let c = Complex::new(i, j);
                    let mut z = Complex::new(0.0, 0.0);
                    let mut color = Color::BLACK;
                    for n in 0..255 {
                        z = z * z + c;
                        if z.norm() >= 2.0 {
                            color = Color::from_rgb8(255 - n, 255 - n, 255 - n);
                            break;
                        }
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
            bytes.push((overall_result[i][j].r * 255.0) as u8);
            bytes.push((overall_result[i][j].g * 255.0) as u8);
            bytes.push((overall_result[i][j].b * 255.0) as u8);
            bytes.push(255);
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

struct RectangleProgram {
    region: Rectangle,
    draw_bounding_box: bool,
}

impl canvas::Program<Message> for RectangleProgram {
    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        if self.draw_bounding_box {
            frame.stroke(
                &canvas::Path::rectangle(
                    Point {
                        x: self.region.x,
                        y: self.region.y,
                    },
                    Size {
                        width: self.region.width,
                        height: self.region.height,
                    },
                ),
                canvas::Stroke::default()
                    .with_color(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    })
                    .with_width(2.0),
            );
        }
        vec![frame.into_geometry()]
    }

    type State = ();
}

fn Solid(a: Color) -> canvas::Style {
    todo!()
}

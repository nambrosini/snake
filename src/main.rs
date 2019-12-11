extern crate rand;

use ggez;
use ggez::event::KeyCode;
use ggez::input::keyboard;
use ggez::conf::WindowMode;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;

use rand::prelude::*;

use std::time::{SystemTime, UNIX_EPOCH};

const WIDTH: i32 = 300;
const HEIGHT: i32 = 300;

const GRID_SIZE: i32 = 10;

#[derive(Clone, Copy)]
struct Body {
    x: i32,
    y: i32,
}

impl Body {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn contains(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }

    fn generate_food(body: &[Self]) -> Self {
        let mut rng = thread_rng();
        loop {
            let mut over = false;
            let food = Body::new((rng.gen::<f64>() * GRID_SIZE as f64) as i32, (rng.gen::<f64>() * GRID_SIZE as f64) as i32);

            for b in body {
                if b.contains(&food) {
                    over = true;
                    break;
                }
            }

            if !over {
                return food;
            }
        }
    }
}

#[derive(PartialEq, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn get_from(actual: &Self, other: &Self) -> (Self, bool) {
        match actual {
            Direction::Up => {
                if other == &Direction::Up || other == &Direction::Down {
                    (Direction::Up, true)
                } else {
                    (other.clone(), false)
                }
            },
            Direction::Down => {
                if other == &Direction::Up || other == &Direction::Down {
                    (Direction::Down, true)
                } else {
                    (other.clone(), false)
                }
            },
            Direction::Left => {
                if other == &Direction::Left || other == &Direction::Right {
                    (Direction::Left, true)
                } else {
                    (other.clone(), false)
                }
            },
            Direction::Right => {
                if other == &Direction::Right || other == &Direction::Left {
                    (Direction::Right, true)
                } else {
                    (other.clone(), false)
                }
            }
        }
    }
}

struct Snake {
    body: Vec<Body>,
    direction: Direction,
    food: Body,
    last_update: u128,
    has_ended: bool,
    direction_changed: bool
}

impl Snake {
    fn new() -> Self {
        let body = vec![Body::new(3, 5), Body::new(3, 6), Body::new(3, 7)];

        Self {
            food: Body::generate_food(&body),
            body,
            direction: Direction::Up,
            last_update: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
            has_ended: false,
            direction_changed: false
        }
    }
}

impl event::EventHandler for Snake {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        if self.has_ended {
            return Ok(())
        }

        let (direction, direction_changed) = if keyboard::is_key_pressed(ctx, KeyCode::W) {
            Direction::get_from(&self.direction, &Direction::Up)
        } else if keyboard::is_key_pressed(ctx, KeyCode::A) {
            Direction::get_from(&self.direction, &Direction::Left)
        } else if keyboard::is_key_pressed(ctx, KeyCode::S) {
            Direction::get_from(&self.direction, &Direction::Down)
        } else if keyboard::is_key_pressed(ctx, KeyCode::D) {
            Direction::get_from(&self.direction, &Direction::Right)
        } else {
            (self.direction.clone(), false)
        };

        if !self.direction_changed {
            self.direction = direction;
            self.direction_changed = direction_changed;
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        if now - self.last_update < 250 {
            return Ok(());
        }

        let (diff_x, diff_y) = match &self.direction {
            Direction::Up => (0, -1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Down => (0, 1)
        };

        let mut body_clone = self.body.clone();
        let (mut last_x, mut last_y) = (body_clone[0].x, body_clone[0].y);

        for i in 0..body_clone.len() {
            if i == 0 {
                body_clone[i].x = body_clone[i].x + diff_x;
                body_clone[i].y = body_clone[i].y + diff_y;
            } else {
                let (tmp_x, tmp_y) = (body_clone[i].x, body_clone[i].y);
                body_clone[i].x = last_x;
                body_clone[i].y = last_y;
                last_x = tmp_x;
                last_y = tmp_y;
            }
        }

        if body_clone[0].contains(&self.food) {
            body_clone.push(Body::new(last_x, last_y));
            self.food = Body::generate_food(&body_clone);
        }

        let head = &body_clone[0];

        if head.x < 0 || head.x >= GRID_SIZE || head.y < 0 || head.y >= GRID_SIZE {
            self.has_ended = true;
        }

        for i in 1..body_clone.len() {
            if body_clone[i].contains(head) {
                self.has_ended = true;
            }
        }

        if !self.has_ended {
            self.body = body_clone;
        }

        self.last_update = now;
        self.direction_changed = false;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        //timer::sleep(Duration::from_millis(250));
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let (cell_w, cell_h) = (WIDTH as i32 / GRID_SIZE, HEIGHT as i32 / GRID_SIZE);

        let body: Vec<ggez::GameResult<graphics::Mesh>> = self
            .body
            .clone()
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let color = if i == 0 {
                    if self.has_ended {
                        graphics::Color::from_rgb(255, 0, 0)
                    } else {
                        graphics::Color::from_rgb(0, 255, 0)
                    }
                } else {
                    graphics::WHITE
                };

                graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    graphics::Rect::new_i32(b.x * cell_w, b.y * cell_h, cell_w, cell_h),
                    color,
                )
            })
            .collect();

        for b in body {
            graphics::draw(ctx, &b.unwrap(), (na::Point2::new(0.0, 0.0),))?;
        }

        let apple = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            na::Point2::new((self.food.x * cell_w + cell_w / 2) as f32, (self.food.y * cell_h + cell_h / 2) as f32),
            (WIDTH / GRID_SIZE / 2) as f32,
            0.1,
            graphics::Color::from_rgb(255, 0, 0),
        )?;

        graphics::draw(ctx, &apple, (na::Point2::new(0.0, 0.0),))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> ggez::GameResult {
    let window_mode = WindowMode {
        width: WIDTH as f32,
        height: HEIGHT as f32,
        maximized: false,
        fullscreen_type: ggez::conf::FullscreenType::Windowed,
        borderless: false,
        min_width: 0.0,
        max_width: 0.0,
        min_height: 0.0,
        max_height: 0.0,
        resizable: false,
    };

    let cb = ggez::ContextBuilder::new("snake", "nambrosini").window_mode(window_mode);
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut Snake::new();
    event::run(ctx, event_loop, state)
}
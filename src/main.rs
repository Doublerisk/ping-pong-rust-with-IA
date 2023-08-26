use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self},
};
use std::io::{self, Write};
use std::time::{Duration, Instant};

const WIDTH: usize = 60;
const HEIGHT: usize = 30;
const PADDLE_HEIGHT: usize = 4;
const PADDLE_WIDTH: usize = 1;
const BALL_START_SPEED: f32 = 5.0;
const BALL_MAX_SPEED: f32 = 25.0;
const BALL_ACCELERATION: f32 = 5.0;
const WINNING_SCORE: u32 = 5;

struct Paddle {
    x: usize,
    y: usize,
}

impl Paddle {
    fn new(x: usize, y: usize) -> Self {
        Paddle { x, y }
    }

    fn move_up(&mut self) {
        if self.y > 0 {
            self.y -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.y + PADDLE_HEIGHT < HEIGHT {
            self.y += 1;
        }
    }
}

struct Ball {
    x: f32,
    y: f32,
    x_speed: f32,
    y_speed: f32,
    last_time: Instant,
}

impl Ball {
    fn new(x: f32, y: f32) -> Self {
        Ball {
            x,
            y,
            x_speed: BALL_START_SPEED,
            y_speed: BALL_START_SPEED,
            last_time: Instant::now(),
        }
    }

    fn update_position(&mut self, paddles: &[Paddle; 2]) {
        let now = Instant::now();
        let time_delta = now.duration_since(self.last_time).as_secs_f32();
        self.last_time = now;

        self.x += self.x_speed * time_delta;
        self.y += self.y_speed * time_delta;

        // Collision detection with paddles
        for paddle in paddles.iter() {
            if self.x >= paddle.x as f32
                && self.x <= (paddle.x + PADDLE_WIDTH) as f32
                && self.y >= paddle.y as f32
                && self.y <= (paddle.y + PADDLE_HEIGHT) as f32
            {
                self.x_speed *= -1.0;
                self.x_speed = (self.x_speed.abs() + BALL_ACCELERATION).min(BALL_MAX_SPEED)
                    * self.x_speed.signum();
            }
        }

        // Collision detection with walls
        if self.y <= 0.0 || self.y >= HEIGHT as f32 - 1.0 {
            self.y_speed *= -1.0;
        }
    }

    fn start_random(&mut self) {
        let now = Instant::now();
        let seed = now.elapsed().as_micros() as u64;
        let mut rng = XorShift::new(seed);
        let direction_x: f32 = rng.gen_range(-1.0, 1.0);
        let direction_y: f32 = rng.gen_range(-1.0, 1.0);
        let magnitude = (direction_x * direction_x + direction_y * direction_y).sqrt();
        self.x_speed = direction_x / magnitude * BALL_START_SPEED;
        self.y_speed = direction_y / magnitude * BALL_START_SPEED;
    }
}

// Simple XorShift random number generator
struct XorShift {
    x: u64,
}

impl XorShift {
    fn new(seed: u64) -> Self {
        XorShift { x: seed }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.x;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.x = x;
        x
    }

    fn gen_range(&mut self, min: f32, max: f32) -> f32 {
        let range = max - min;
        min + (self.next() as f32 % range)
    }
}

fn draw(paddles: &[Paddle; 2], ball: &Ball, score: &[u32; 2]) {
    print!("\x1B[2J\x1B[1;1H");

    // Draw paddles
    for paddle in paddles.iter() {
        for i in 0..PADDLE_HEIGHT {
            print!("\x1B[{};{}H█", paddle.y + i + 1, paddle.x + 1);
        }
    }

    // Draw ball
    print!("\x1B[{};{}HO", ball.y as usize + 1, ball.x as usize + 1);

    // Draw scores
    println!(
        "\x1B[{};{}HPlayer 1: {}",
        HEIGHT + 1,
        WIDTH / 2 - 10,
        score[0]
    );
    println!(
        "\x1B[{};{}HPlayer 2: {}",
        HEIGHT + 1,
        WIDTH / 2 + 5,
        score[1]
    );
    io::stdout().flush().unwrap();
}

fn main() {
    let mut paddles = [
        Paddle::new(2, HEIGHT / 2 - PADDLE_HEIGHT / 2),
        Paddle::new(WIDTH - 3, HEIGHT / 2 - PADDLE_HEIGHT / 2),
    ];

    let mut ball = Ball::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
    let mut score = [0, 0];

    terminal::enable_raw_mode().unwrap();

    // Start ball movement in a random direction
    ball.start_random();

    loop {
        if let Ok(true) = event::poll(Duration::from_millis(33)) {
            if let event::Event::Key(key_event) = event::read().unwrap() {
                match key_event {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => break,
                    KeyEvent {
                        code: KeyCode::Up,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => paddles[1].move_up(),
                    KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => paddles[1].move_down(),
                    KeyEvent {
                        code: KeyCode::Char('w'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => paddles[0].move_up(),
                    KeyEvent {
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => paddles[0].move_down(),
                    _ => (),
                }
            }
        }

        ball.update_position(&paddles);
        draw(&paddles, &ball, &score);

        // Check if ball goes out of bounds and update score
        if ball.x <= 0.0 {
            score[1] += 1;
            reset_ball(&mut ball);
        } else if ball.x >= WIDTH as f32 - 1.0 {
            score[0] += 1;
            reset_ball(&mut ball);
        }

        // Check if any player has won
        if score[0] >= WINNING_SCORE || score[1] >= WINNING_SCORE {
            break;
        }
    }

    terminal::disable_raw_mode().unwrap();

    // Print game over message
    println!("\nGame Over!");
    if score[0] > score[1] {
        println!("Player 1 wins!");
    } else {
        println!("Player 2 wins!");
    }
}

fn reset_ball(ball: &mut Ball) {
    ball.x = WIDTH as f32 / 2.0;
    ball.y = HEIGHT as f32 / 2.0;
    ball.start_random();
}

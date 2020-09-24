use std::collections::HashMap;

use lazy_static::lazy_static;
use maplit::hashmap;
use rand::Rng;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeStruct;
use crate::screen::{COLOR_PAIR_SNK1, COLOR_PAIR_SNK2};

lazy_static! {
    static ref FORBIDDEN_DIRECTIONS: HashMap<&'static DIRECTION, DIRECTION> = hashmap! {
        &DIRECTION::Up => DIRECTION::Down,
        &DIRECTION::Down => DIRECTION::Up,
        &DIRECTION::Right => DIRECTION::Left,
        &DIRECTION::Left => DIRECTION::Right,
    };
}

#[derive(Clone)]
pub enum MODE {
    Single,
    Server,
    Client,
}

pub struct Game {
    mode: MODE,
    game_area_height: i32,
    game_area_width: i32,
    game_area_start_y: i32,
    game_area_start_x: i32,
    pub clear1_buffer: Vec<Field>,
    pub clear2_buffer: Vec<Field>,
    pub food: Field,
    draw_food: fn(&i32, &i32),
    draw_block: fn(&i32, &i32, &i16),
    clear_field: fn(&i32, &i32),
}

impl Game {
    pub fn new(height: i32, width: i32,
               draw_food: fn(&i32, &i32),
               draw_block: fn(&i32, &i32, &i16),
               clear_field: fn(&i32, &i32)) -> Self {
        Game {
            mode: MODE::Single,
            game_area_height: height,
            game_area_width: width,
            game_area_start_y: 0,
            game_area_start_x: 0,
            clear1_buffer: vec![],
            clear2_buffer: vec![],
            food: Field{y: 0, x: 0},
            draw_food,
            draw_block,
            clear_field,
        }
    }

    pub fn draw_snake(&mut self, buffer1: Vec<Field>, buffer2: Vec<Field>) {
        for field in &self.clear1_buffer {
            (self.clear_field)(&(field.y + self.game_area_start_y),
                               &(field.x + self.game_area_start_x));
        }
        for field in &buffer1 {
            (self.draw_block)(&(field.y + self.game_area_start_y),
                              &(field.x + self.game_area_start_x), &COLOR_PAIR_SNK1);
        }
        self.clear1_buffer = buffer1;

        for field in &self.clear2_buffer {
            (self.clear_field)(&(field.y + self.game_area_start_y),
                               &(field.x + self.game_area_start_x));
        }
        for field in &buffer2 {
            (self.draw_block)(&(field.y + self.game_area_start_y),
                              &(field.x + self.game_area_start_x), &COLOR_PAIR_SNK2);
        }
        self.clear2_buffer = buffer2;
    }

    pub fn transform_coords(&self, buffer: &mut Vec<Field>) {
        for mut field in buffer {
            field.y = field.y - self.game_area_start_y;
            field.x = field.x - self.game_area_start_x;
        }
    }

    pub fn get_mode(&self) -> MODE {
        self.mode.clone()
    }

    pub fn set_mode(&mut self, mode: MODE) {
        self.mode = mode.clone();
    }

    pub fn set_start(&mut self, start_y: i32, start_x: i32) {
        self.game_area_start_y = start_y;
        self.game_area_start_x = start_x;
    }

    pub fn check_food(&mut self, snake: &mut Snake) {
        if snake.body[0] == self.food {
            self.init_food(&snake.body, &snake.body); // fix this...
        }
        else {
            let tail = snake.body.pop().unwrap();
            (self.clear_field)(&tail.y, &tail.x);
        }
    }

    pub fn init_food (&mut self, forbidden1: &Vec<Field>, forbidden2: &Vec<Field>) {
        self.food = self.new_food(forbidden1, forbidden2);
        (self.draw_food)(&self.food.y, &self.food.x);
    }

    pub fn set_food (&mut self, y: i32, x: i32) {
        (self.clear_field)(&(self.food.y + self.game_area_start_y),
                           &(self.food.x + self.game_area_start_x));
        self.food.y = y;
        self.food.x = x;
        (self.draw_food)(&(self.food.y + self.game_area_start_y),
                         &(self.food.x + self.game_area_start_x));
    }

    pub fn get_food (&mut self) -> (i32, i32) {
        (self.food.y, self.food.x)
    }

    pub fn get_food_win (&mut self) -> (i32, i32) {
        (self.food.y - self.game_area_start_y, self.food.x - self.game_area_start_x)
    }

    pub fn new_food (&self, forbidden1: &Vec<Field>, forbidden2: &Vec<Field>) -> Field {
        // Food cannot appear on snake!
        let mut field: Field;
        loop {
            field = Field{y: rand::thread_rng().gen_range(self.game_area_start_y + 1,
                                                          self.game_area_start_y + self.game_area_height - 1),
                          x: rand::thread_rng().gen_range(self.game_area_start_x + 1,
                                                          self.game_area_start_x + self.game_area_width - 1)};
            if !(forbidden1.contains(&field)) && !(forbidden2.contains(&field)) {
               break;
            }
        };
        field
    }

    pub fn check_collisions(&self, snakes: &Vec<Snake>) -> bool {
        let mut self_collision = false;
        for snake in snakes {
            let head = snake.body.get(0).unwrap();
            for element in snake.body[1..].iter() {
                if element == head {
                    self_collision = true;
                }
            }
        }

        let mut game_area_collision = false;
        for snake in snakes {
            if snake.body[0].y == self.game_area_start_y ||
                snake.body[0].y == self.game_area_start_y + self.game_area_height - 1 ||
                snake.body[0].x == self.game_area_start_x ||
                snake.body[0].x == self.game_area_start_x + self.game_area_width - 1 {
                game_area_collision = true;
            }
        }

        let mut snakes_collision = false;
        if snakes[0].body.contains(&snakes[1].body[0]) {
            snakes_collision = true;
        }
        if snakes[1].body.contains(&snakes[0].body[0]) {
            snakes_collision = true;
        }

        self_collision || game_area_collision || snakes_collision
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum DIRECTION {
    Down,
    Up,
    Right,
    Left,
}

#[derive(Clone, Deserialize)]
pub struct Field {
    pub(crate) y: i32,
    pub(crate) x: i32,
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.y == other.y && self.x == other.x
    }
}

impl Serialize for Field {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        // 2 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Field", 2)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("x", &self.x)?;
        state.end()
    }
}

pub struct Snake {
    pub(crate) body: Vec<Field>,
    pub(crate) direction: DIRECTION,
    draw_block: fn(&i32, &i32, &i16),
    color: i16,
}

impl Snake {
    pub fn new(head_y: i32, head_x: i32, direction: DIRECTION,
               draw_block: fn(&i32, &i32, &i16), color: i16) -> Self {
        let mut snake = Snake {
            body: Vec::from(vec![Field{y: head_y, x: head_x}]),
            direction,
            draw_block,
            color,
        };
        if snake.direction == DIRECTION::Right {
            snake.body.push(Field{y: head_y, x: head_x - 1});
        }
        else {
            snake.body.push(Field{y: head_y, x: head_x + 1});
        }
        snake
    }

    pub fn change_direction(&mut self, direction: &DIRECTION) {
        if let Some(forbidden_direction) = FORBIDDEN_DIRECTIONS.get(&self.direction) {
            if forbidden_direction != direction {
                self.direction = direction.clone();
            }
        }
    }

    pub fn get_head(&self) -> (i32, i32) {
        (self.body[0].y, self.body[0].x)
    }

    pub fn move_snake(&mut self) {
        let new_head = Field{y: self.body[0].y, x: self.body[0].x};
        self.body.insert(0, new_head);

        match self.direction {
            DIRECTION::Down => {self.body[0].y += 1},
            DIRECTION::Up => {self.body[0].y -= 1},
            DIRECTION::Left => {self.body[0].x -= 1},
            DIRECTION::Right => {self.body[0].x += 1},
        }
        (self.draw_block)(&self.body[0].y, &self.body[0].x, &self.color);
    }
}

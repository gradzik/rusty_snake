extern crate bincode;
extern crate ncurses;
extern crate serde;

use std::collections::HashMap;
use std::convert::TryInto;
use std::io;
use std::thread::sleep;
use std::time::{Duration, Instant};

use clap::{App, Arg};
use lazy_static::lazy_static;
use maplit::hashmap;
use ncurses::*;
use serde::{Deserialize, Serialize};

use crate::game::{DIRECTION, Field, Game, MODE, Snake};
use crate::network::{init_network, send_endgame, UdpFrame};
use crate::screen::{create_status_window, create_game_area, create_ending_window, destroy_window,
                    GAME_AREA_HEIGHT, GAME_AREA_WIDTH, update_score, init_window_colors,
                    COLOR_PAIR_FOOD, print_ending_info};

mod network;
mod game;
mod screen;

static REFRESH_TIMEOUT: i32 = 100;

lazy_static! {
    static ref DIRECTIONS: HashMap<&'static i32, DIRECTION> = hashmap! {
        &KEY_DOWN => DIRECTION::Down,
        &KEY_UP => DIRECTION::Up,
        &KEY_LEFT => DIRECTION::Left,
        &KEY_RIGHT => DIRECTION::Right,
    };
}

#[derive(Serialize, Deserialize)]
enum COMMANDS {
    Connect,
    Endgame,
    Key(i32),
    ServerData(UdpFrame),
}

fn main() {
    let matches = App::new("Rusty Snake")
        .version("0.1.0")
        .author("Rafal Grad <r.grad@wp.pl>")
        .about("A simple Snake game written in Rust with multiplayer.")
        .arg(Arg::with_name("connect")
            .short("c")
            .long("conn")
            .takes_value(true)
            .help("Address of Snake server to connect"))

        .arg(Arg::with_name("server")
            .short("s")
            .long("server")
            .help("Run as multiplayer game server")
            .requires("port"))

        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .takes_value(true)
            .help("Server port number"))

        .get_matches();

    /* Initialize game data. */
    let mut game = Game::new(GAME_AREA_HEIGHT, GAME_AREA_WIDTH, print_food, print_block, print_space);
    let mut snakes = vec![];
    let mut udp_buffer = [0; 10000];

    /* Get input from user. */
    let address = matches.value_of("connect").unwrap_or("0.0.0.0:10000");
    let server_port = matches.value_of("port").unwrap_or("10000");
    let is_server = matches.is_present("server");

    /*  Set game mode based on user input. */
    if is_server {
        game.set_mode(MODE::Server);
    }
    else if matches.is_present("connect") {
        game.set_mode(MODE::Client);
    }
    else {
        game.set_mode(MODE::Single);
    }

    /* Initialize networ connection. */
    let socket = init_network(&game, &server_port, &address);

    /* Initialising ncurses. */
    init_ncurses();

    /* Initialising game windows. */
    init_window_colors();
    let status_window = create_status_window();
    let game_window = create_game_area(&mut game, &mut snakes);

    /* Initialising a game. */
    let mut endgame = false;
    let mut pressed_key = KEY_LEFT;
    let mut received_command = KEY_LEFT;
    let mut direction_from_key = DIRECTION::Left;
    let mut direction_from_udp = DIRECTION::Left;
    let mut direction_from_alg = DIRECTION::Left;

    match game.get_mode() {
        MODE::Server => {
            game.init_food(&snakes[0].body, &snakes[1].body);
            loop {
                match socket.recv_from(&mut udp_buffer) {
                    Ok(n) => {
                        // n.1 -> data source address
                        socket.connect(n.1).expect("connect function failed");

                        let deserialized: COMMANDS = bincode::deserialize(&udp_buffer[0..(n.0)]).unwrap();
                        match deserialized {
                            COMMANDS::Connect => { },
                            _ => {}
                        }

                        let mut snake1 = snakes[0].body.to_vec();
                        let mut snake2 = snakes[1].body.to_vec();
                        game.transform_coords(&mut snake1);
                        game.transform_coords(&mut snake2);
                        let food = game.get_food_win();
                        let frame = UdpFrame{snake1, snake2, food: Field{y: food.0, x: food.1} };

                        let serialized = bincode::serialize(&COMMANDS::ServerData(frame)).unwrap();
                        socket.send(&serialized).expect("couldn't send message");

                        break;
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => { }
                    Err(e) =>  {
                        println!("encountered IO error: {}", e)
                    },
                }
                sleep(Duration::from_millis(10));
            }
        }
        MODE::Client => {
            loop {
                match socket.recv_from(&mut udp_buffer) {
                    Ok(n) => {
                        let deserialized: COMMANDS = bincode::deserialize(&udp_buffer[0..(n.0)]).unwrap();
                        match deserialized {
                            COMMANDS::ServerData(frame) => {
                                game.set_food(frame.food.y, frame.food.x);
                                update_score(status_window, &socket, &game.get_mode(),
                                             &(frame.snake1.len()).try_into().unwrap(),
                                             &(frame.snake2.len()).try_into().unwrap());
                                game.draw_snake(frame.snake1, frame.snake2);
                            }
                            _ => {}
                        }
                        break;
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => { }
                    Err(e) => {
                        println!("encountered IO error: {}", e)
                    },
                }
            }
        }
        MODE::Single => {
            game.init_food(&snakes[0].body, &snakes[1].body);
        }
    }

    loop
    {
        let now = Instant::now();

        /* Loop for checking inputs and timeout. */
        while (now.elapsed().as_millis() as i32) < REFRESH_TIMEOUT {

            pressed_key = getch();
            if pressed_key != -1 {
                if let Some(direction) = DIRECTIONS.get(&pressed_key) {
                    direction_from_key = direction.clone();
                    break;
                }
            }

            match socket.recv_from(&mut udp_buffer) {
                Ok(n) => {
                    let deserialized: COMMANDS = bincode::deserialize(&udp_buffer[0..(n.0)]).unwrap();
                    match deserialized {
                        COMMANDS::Key(key) => {
                            received_command = key;
                        },
                        COMMANDS::ServerData(frame) => {
                            game.set_food(frame.food.y, frame.food.x);
                            update_score(status_window, &socket, &game.get_mode(),
                                         &(frame.snake1.len()).try_into().unwrap(),
                                         &(frame.snake2.len()).try_into().unwrap());
                            game.draw_snake(frame.snake1, frame.snake2);
                        }
                        COMMANDS::Endgame => {
                            endgame = true;
                        }
                        _ => {println!("encountered IO error: ")}
                    }
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    mvwaddstr(stdscr(), LINES() - 22, 0, "BLAD POLACZENIA");
                    println!("{}", e);
                },
            }

            if let Some(direction) = DIRECTIONS.get(&received_command) {
                direction_from_udp = direction.clone();
            }

            let (head_y, head_x) = snakes[1].get_head();
            let (target_y, target_x) = game.get_food();
            direction_from_alg = calculate_direction(&snakes[1], &target_y, &target_x, &head_y, &head_x);

            if pressed_key == 'q' as i32 {
                endgame = true;
                break;
            };
            sleep(Duration::from_millis(10));
        }

        /* Precess inputs from key, network. */
        match game.get_mode() {
            MODE::Client => {
                let serialized = bincode::serialize(&COMMANDS::Key(pressed_key)).unwrap();
                socket.send(&serialized).expect("couldn't send message");
            }
            MODE::Server => {
                if let Some(snake) = snakes.get_mut(0) {
                    snake.change_direction(&direction_from_key)
                }
                if let Some(snake) = snakes.get_mut(1) {
                    snake.change_direction(&direction_from_udp)
                }

                let mut snake1 = snakes[0].body.to_vec();
                let mut snake2 = snakes[1].body.to_vec();
                game.transform_coords(&mut snake1);
                game.transform_coords(&mut snake2);
                let food = game.get_food_win();
                let frame = UdpFrame{snake1, snake2, food: Field{y: food.0, x: food.1} };

                let serialized = bincode::serialize(&COMMANDS::ServerData(frame)).unwrap();
                socket.send(&serialized).expect("couldn't send message");
            },
            MODE::Single => {
                if let Some(snake) = snakes.get_mut(0) {
                    snake.change_direction(&direction_from_key)
                }
                if let Some(snake) = snakes.get_mut(1) {
                    snake.change_direction(&direction_from_alg)
                }
            },
        }

        /* Snake moves in Single player and Server mode. */
        match game.get_mode() {
            MODE::Client => {},
            _ => {
                for mut snake in &mut snakes {
                    snake.move_snake();
                    game.check_food(&mut snake);
                }
                if game.check_collisions(&snakes) {
                    endgame = true;
                }

                update_score(status_window, &socket, &game.get_mode(),
                             &(snakes[0].body.len()).try_into().unwrap(),
                             &(snakes[1].body.len()).try_into().unwrap());
            }
        }

        /* End game and print information to user. */
        if endgame {
            werase(stdscr());
            refresh();

            destroy_window(game_window);
            destroy_window(status_window);
            let win = create_ending_window();

            match game.get_mode() {
                MODE::Single => {
                    let score1: &i32 = &(snakes[0].body.len()).try_into().unwrap();
                    let score2: &i32 = &(snakes[1].body.len()).try_into().unwrap();
                    print_ending_info(win, &score1, &score2);
                },
                MODE::Server => {
                    send_endgame(&socket);

                    let score1: &i32 = &(snakes[0].body.len()).try_into().unwrap();
                    let score2: &i32 = &(snakes[1].body.len()).try_into().unwrap();
                    print_ending_info(win, &score1, &score2);
                },
                MODE::Client => {
                    let score1: &i32 = &(game.clear2_buffer.len()).try_into().unwrap();
                    let score2: &i32 = &(game.clear1_buffer.len()).try_into().unwrap();
                    print_ending_info(win, &score1, &score2);
                },
            }

            while getch() != 'q' as i32 {
                sleep(Duration::from_millis(10));
            }
            destroy_window(win);
            break;
        }

        wrefresh(status_window);
        wrefresh(game_window);
    }
    endwin();
}

fn calculate_direction(snake: &Snake, target_y: &i32, target_x: &i32, head_y: &i32, head_x: &i32) -> DIRECTION {
    let mut direction = DIRECTION::Left;
    let mut helper_flag = false;

    if head_x > target_x {
        direction = DIRECTION::Left;
        if snake.direction == DIRECTION::Right {
            helper_flag = true;
        };
    }
    if head_x < target_x {
        direction = DIRECTION::Right;
        if snake.direction == DIRECTION::Left {
            helper_flag = true;
        };
    }
    if head_x == target_x || helper_flag == true {
        if head_y < target_y {
            direction = DIRECTION::Down;
        };
        if head_y > target_y {
            direction = DIRECTION::Up;
        };
    }
    direction
}

fn init_ncurses() {
    initscr();

    keypad(stdscr(), true);
    timeout(0);
    noecho();

    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    refresh();

    addstr("Use the arrow keys to move");
    mvwaddstr(stdscr(), LINES() - 1, 0, "Press 'q' to exit");
    start_color();
}

fn print_block(y: &i32, x: &i32, color: &i16) {
    attr_on(COLOR_PAIR(*color));
    mvaddch(*y, *x, ACS_CKBOARD());
}

fn print_food(y: &i32, x: &i32) {
    attr_on(COLOR_PAIR(COLOR_PAIR_FOOD));
    mvaddch(*y, *x, ACS_DIAMOND());
}

fn print_space(y: &i32, x: &i32) {
    let ch = ' ' as chtype;
    mvaddch(*y, *x, ch);
}

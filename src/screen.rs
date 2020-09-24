use crate::game::{Snake, DIRECTION, Game, MODE};
use crate::{print_block};
use ncurses::*;
use std::net::UdpSocket;

pub(crate) static GAME_AREA_HEIGHT: i32 = 20;
pub(crate) static GAME_AREA_WIDTH: i32 = 60;
static GAME_ENDING_HEIGHT: i32 = 10;
static GAME_ENDING_WIDTH: i32 = 30;
static STATUS_AREA_WIDTH: i32 = 30;

pub fn print_ending_info (win: WINDOW, score1: &i32, score2: &i32) {
    mvwaddstr(win, 2, 10, "GAME OVER");
    mvwaddstr(win, 7, 6, "Press 'q' to exit.");
    if score1 > score2 { mvwaddstr(win, 3, 11, "YOU WIN!"); }
    else if score1 == score2 { mvwaddstr(win, 3, 8, "WE HAVE A TIE!"); }
    else { mvwaddstr(win, 3, 10, "YOU LOSE!"); }
    mvwaddstr(win, 5, 3, &*format!("Player1: {}   Player2: {}", score1, score2));
    wrefresh(win);
}

pub fn create_ending_window() -> WINDOW {
    /* Get the screen bounds. */
    let mut screen_max_y = 0;
    let mut screen_max_x = 0;
    getmaxyx(stdscr(), &mut screen_max_y, &mut screen_max_x);

    /* Start in the center. */
    let start_y = (screen_max_y - GAME_ENDING_HEIGHT) / 2;
    let start_x = (screen_max_x - GAME_ENDING_WIDTH) / 2;

    let win = newwin(GAME_ENDING_HEIGHT, GAME_ENDING_WIDTH, start_y, start_x);
    box_(win, 0, 0);
    wrefresh(win);
    win
}

pub fn update_score(win: WINDOW, socket: &UdpSocket, mode: &MODE, player1: &i32, player2: &i32) {
    wattr_on(win, COLOR_PAIR(COLOR_PAIR_FOOD));

    match mode {
        MODE::Server => {
            mvwaddstr(win, 6, 1,  "    Multiplayer server      ");
            let local_address = socket.local_addr().unwrap();
            mvwaddstr(win, 9, 5, &*format!("{}", local_address));
            let client_address = socket.peer_addr().unwrap();
            mvwaddstr(win, 12, 5, &*format!("{}", client_address));
        },
        MODE::Client => {
            mvwaddstr(win, 6, 1,  "    Multiplayer client      ");
            let client_address = socket.peer_addr().unwrap();
            mvwaddstr(win, 9, 5, &*format!("{}", client_address));
            let local_address = socket.local_addr().unwrap();
            mvwaddstr(win, 12, 5, &*format!("{}", local_address));
        }
        _ => {}
    }

    wattr_on(win, COLOR_PAIR(COLOR_PAIR_SNK1_SC));
    mvwaddstr(win, 16, 1, &*format!("    Player 1 score:  {}     ", player1));
    wattr_on(win, COLOR_PAIR(COLOR_PAIR_SNK2_SC));
    mvwaddstr(win, 17, 1, &*format!("    Player 2 score:  {}     ", player2));
}

pub fn create_status_window() -> WINDOW {
    /* Get the screen bounds. */
    let mut screen_max_y = 0;
    let mut screen_max_x = 0;
    getmaxyx(stdscr(), &mut screen_max_y, &mut screen_max_x);

    /* Start in the center. */
    let start_y = (screen_max_y - GAME_AREA_HEIGHT) / 2;
    let start_x = (screen_max_x - GAME_AREA_WIDTH - STATUS_AREA_WIDTH) / 2 - 2;

    let win = newwin(GAME_AREA_HEIGHT, STATUS_AREA_WIDTH, start_y, start_x);
    box_(win, 0, 0);

    mvwaddstr(win, 1, 1,  "                            ");
    mvwaddstr(win, 2, 1,  "        RUSTY SNAKE         ");
    mvwaddstr(win, 4, 1,  "----------------------------");
    mvwaddstr(win, 6, 1,  "       Single player        ");
    mvwaddstr(win, 8, 1,  "      Server address:       ");
    mvwaddstr(win, 9, 1,  "          ------            ");
    mvwaddstr(win, 11, 1, "      Client address:       ");
    mvwaddstr(win, 12, 1, "          ------            ");
    mvwaddstr(win, 14, 1, "----------------------------");

    wattr_on(win, COLOR_PAIR(COLOR_PAIR_SNK1_SC));
    mvwaddstr(win, 16, 1, "    Player 1 score:         ");
    wattr_on(win, COLOR_PAIR(COLOR_PAIR_SNK2_SC));
    mvwaddstr(win, 17, 1, "    Player 2 score:         ");

    wrefresh(win);
    win
}

pub fn create_game_area(game: &mut Game, snakes: &mut Vec<Snake>) -> WINDOW {
    /* Get the screen bounds. */
    let mut screen_max_y = 0;
    let mut screen_max_x = 0;
    getmaxyx(stdscr(), &mut screen_max_y, &mut screen_max_x);

    /* Start in the center. */
    let start_y = (screen_max_y - GAME_AREA_HEIGHT) / 2;
    let start_x = (screen_max_x - GAME_AREA_WIDTH + STATUS_AREA_WIDTH) / 2 + 2;
    game.set_start(start_y, start_x);

    snakes.push(Snake::new(start_y + GAME_AREA_HEIGHT/2, start_x + 2,
                           DIRECTION::Right, print_block, COLOR_PAIR_SNK1));
    snakes.push(Snake::new(start_y + GAME_AREA_HEIGHT/2, start_x + GAME_AREA_WIDTH - 3,
                           DIRECTION::Left, print_block, COLOR_PAIR_SNK2));

    let win = newwin(GAME_AREA_HEIGHT, GAME_AREA_WIDTH, start_y, start_x);
    box_(win, 0, 0);
    wrefresh(win);
    win
}

pub(crate) static COLOR_PAIR_FOOD: i16 = 1;
static COLOR_PAIR_SNK1_SC: i16 = 2;
static COLOR_PAIR_SNK2_SC: i16 = 3;
pub(crate) static COLOR_PAIR_SNK1: i16 = 4;
pub(crate) static COLOR_PAIR_SNK2: i16 = 5;

pub fn init_window_colors() {
    init_pair(COLOR_PAIR_FOOD, COLOR_WHITE, COLOR_BLACK);
    init_pair(COLOR_PAIR_SNK1_SC, COLOR_BLACK, COLOR_RED);
    init_pair(COLOR_PAIR_SNK2_SC, COLOR_BLACK, COLOR_BLUE);
    init_pair(COLOR_PAIR_SNK1, COLOR_RED, COLOR_BLACK);
    init_pair(COLOR_PAIR_SNK2, COLOR_BLUE, COLOR_BLACK);
}


pub fn destroy_window(win: WINDOW)
{
    let ch = ' ' as chtype;
    wborder(win, ch, ch, ch, ch, ch, ch, ch, ch);
    wclear(win);
    wrefresh(win);
    delwin(win);
}

use std::rc::Rc;

use gloo::storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use battleship_core::{GameState, Ship};


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub game: GameState,
    pub is_ready: bool,
}

pub enum Action {
    PlaceShips([Ship; 5], String),
    GameReset(),
}

impl Reducible for State {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Action::PlaceShips(ships, name) => {
                LocalStorage::set(name, ships);
                State {
                    game: GameState {
                        ships:ships,
                        salt: 0xDEADBEEF,
                    },
                    is_ready: true,
                }
                .into()

            }
            Action::GameReset() => State::reset().into(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {

            game: GameState::new(),
            is_ready: false,
        }

    }
}

impl State {
    pub fn reset() -> State {
        State {
            game: LocalStorage::get("some name").unwrap_or(GameState::new()),
            is_ready: false,
        }

    }
}

#[macro_use]
extern crate rocket;

use rocket::response::Responder;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use sudoku_lib::{Board, Coord, Row, Zone};

/// Result of attempting to solve the sudoku puzzle.
#[derive(Error, Debug, Responder)]
enum SolveFailure {
    #[response(status = /* Unprocessable Entity */ 422)]
    #[error("{0}")]
    BadRequest(String),
    #[response(status = /* Conflict */ 409)]
    #[error("{0}")]
    NoSolution(String),
}

#[derive(Serialize, Deserialize)]
struct Solution {}

#[post("/api/sudoku/solve", data = "<board>")]
fn solve(board: Json<Vec<Vec<Option<u8>>>>) -> Result<Json<Vec<Vec<u8>>>, SolveFailure> {
    let raw_board = board.into_inner();
    if raw_board.len() != 9 {
        return Err(SolveFailure::BadRequest(format!(
            "Expected 9 rows, got {}",
            raw_board.len()
        )));
    }
    if let Some((i, wrong)) = raw_board.iter().enumerate().find(|(_, row)| row.len() != 9) {
        return Err(SolveFailure::BadRequest(format!(
            "Expected 9 colums, got {} on row {}",
            wrong.len(),
            i
        )));
    }
    let mut board = Board::new();
    for (y, row) in raw_board.iter().enumerate() {
        for (x, &col) in row.iter().enumerate() {
            if let Some(val) = col {
                if !(1..=9).contains(&val) {
                    return Err(SolveFailure::BadRequest(format!(
                        "Values must be in range [1, 9], got {} on row {} column {}",
                        val, y, x
                    )));
                }
                board.specify(Coord::new(y, x), val);
            }
        }
    }
    if let Some(solution) = board.solve() {
        let mut res = Vec::with_capacity(Board::SIZE);
        for r in Row::all() {
            let mut row = Vec::with_capacity(Row::SIZE as usize);
            for coord in r.indexes() {
                row.push(
                    solution
                        .get(coord)
                        .expect("Solution was expected to have all cells with single known values"),
                );
            }
            res.push(row);
        }
        Ok(Json(res))
    } else {
        Err(SolveFailure::NoSolution("No solution found".to_string()))
    }
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![solve])
}

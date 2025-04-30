use std::num::ParseIntError;

use anyhow::Error;
use rand::TryRngCore;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepType {
    New,                  // First time we're seeing this block
    Undo = 2,             // We are undoing this block (it was came as New previously)
    Irreversible = 16,    // This block is now final and cannot be 'Undone' anymore (irreversible)
    NewIrreversible = 17, // NEW | IRREVERSIBLE
    Stalled = 32,         // This block passed the LIB and is definitely forked out
    All = 68,             // StepNew | StepUndo | StepIrreversible | StepStalled
}

const NEW: i32 = 1;
const UNDO: i32 = 2;
const IRREVERSIBLE: i32 = 16;
const NEW_IRREVERSIBLE: i32 = NEW | IRREVERSIBLE;
const STALLED: i32 = 32;
const ALL: i32 = NEW | UNDO | IRREVERSIBLE | STALLED;

pub fn from_str(part: &str) -> Result<StepType, Error> {
    match part.parse::<i32>() {
        Ok(s) => match s {
            NEW => Ok(StepType::New),
            UNDO => Ok(StepType::Undo),
            IRREVERSIBLE => Ok(StepType::Irreversible),
            NEW_IRREVERSIBLE => Ok(StepType::NewIrreversible),
            STALLED => Ok(StepType::Stalled),
            ALL => Ok(StepType::All),
            _ => panic!("Invalid step type: {}", part),
        },
        Err(e) => Err(Error::msg(format!("Invalid step type: {}", part))),
    }
}

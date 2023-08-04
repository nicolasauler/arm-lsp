use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Instruction {
    id: String,
    names: Vec<String>,
    operation: Operation,
}

#[derive(Serialize, Deserialize, Debug)]
struct Operation {
    lines: Vec<String>,
}

fn populate_hashmap() -> Result<HashMap<String, Instruction>, Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string("all.json")?;
    let instructions: Vec<Instruction> = serde_json::from_str(&file)?;

    let mut instructions_map = HashMap::new();
    for instruction in instructions {
        instructions_map.insert(instruction.names[0].clone(), instruction);
    }

    Ok(instructions_map)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instructions_map = populate_hashmap()?;

    Ok(())
}

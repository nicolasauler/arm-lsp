use std::error::Error;
use std::fs::File;
use std::io::BufRead;

use lsp_types::request::{GotoDefinition, HoverRequest};
use lsp_types::{
    Hover, HoverContents, HoverProviderCapability, InitializeParams, Location, MarkedString, OneOf,
    Position, Range, ServerCapabilities, TextDocumentPositionParams, Url,
};

use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Instruction {
    id: String,
    names: Vec<String>,
    operation: Operation,
    symbols: Symbols,
    summary: Summary,
}

#[derive(Serialize, Deserialize, Debug)]
struct Operation {
    lines: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Symbols {
    lines: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Summary {
    lines: Vec<String>,
}

fn populate_hashmap() -> Result<HashMap<String, Vec<Instruction>>, Box<dyn Error + Sync + Send>> {
    let file = std::fs::read_to_string("all.json")?;
    let instructions: Vec<Instruction> = serde_json::from_str(&file)?;

    let mut instructions_map: HashMap<String, Vec<Instruction>> = HashMap::new();
    for instruction in instructions {
        // if instruction name is already in hashmap, append the new instruction to the existing one in the list
        match instructions_map.get_mut(&instruction.names[0].clone().to_uppercase()) {
            Some(existing_instruction) => {
                existing_instruction.push(instruction);
            }
            None => {
                instructions_map.insert(
                    instruction.names[0].clone().to_uppercase(),
                    vec![instruction],
                );
            }
        }
    }

    Ok(instructions_map)
}

fn get_instructions<'a>(
    instructions_map: &'a HashMap<String, Vec<Instruction>>,
    instruction_name: &str,
) -> Option<&'a Vec<Instruction>> {
    instructions_map.get(instruction_name)
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("starting generic LSP server");

    let instructions_map = populate_hashmap()?;

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    })
    .unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params, &instructions_map)?;
    io_threads.join()?;

    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
    instructions_map: &HashMap<String, Vec<Instruction>>,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    eprintln!("starting example main loop");
    for msg in &connection.receiver {
        eprintln!("got msg: {msg:?}");
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                eprintln!("got request: {req:?}");
                match cast::<GotoDefinition>(req.clone()) {
                    Ok((id, params)) => {
                        eprintln!("got gotoDefinition request #{id}: {params:?}");
                        let resp =
                            match text_document_definition(&params.text_document_position_params) {
                                Some(result) => {
                                    let result = serde_json::to_value(&result).unwrap();
                                    Response {
                                        id,
                                        result: Some(result),
                                        error: None,
                                    }
                                }
                                None => Response {
                                    id,
                                    result: None,
                                    error: None,
                                },
                            };
                        eprintln!("sending response: {resp:?}");
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };
                match cast::<HoverRequest>(req) {
                    Ok((id, params)) => {
                        eprintln!("got hover request #{id}: {params:?}");

                        let word_at_cursor =
                            get_word_at_cursor_from_file(&params.text_document_position_params);
                        let word_at_cursor = word_at_cursor.to_uppercase();
                        eprintln!("word at cursor: {word_at_cursor}");

                        let lsp_types = Hover {
                            contents: HoverContents::Scalar(MarkedString::String(
                                // print operation, summary and symbols for each instruction in the
                                // vector of instructions returned by get_instructions
                                match get_instructions(&instructions_map, &word_at_cursor) {
                                    Some(vec) => vec
                                        .iter()
                                        .map(|instruction| {
                                            instruction.operation.lines.join("\n")
                                                + "\n\n"
                                                + &instruction.summary.lines.join("\n")
                                                + "\n\n"
                                                + &instruction.symbols.lines.join("\n")
                                                + "\n\n"
                                        })
                                        .collect::<Vec<String>>()
                                        .join("\n\n"),
                                    None => "Not an instruction\nor\nNo information available"
                                        .to_string(),
                                },
                            )),
                            range: None,
                        };
                        let result = Some(lsp_types);
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        eprintln!("sending response: {resp:?}");
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }
            Message::Notification(not) => {
                eprintln!("got notification: {not:?}");
            }
        }
    }
    Ok(())
}

// implement go to definition for the language server
// based on current source position, find the word at the cursor
// then find definition and return its position
//
// example file to test: test.asm
// contents:
// L0: mov ax, 0x1234
//     mov bx, 0x5678
//     bnz L0
//
// if you send a go to definition on L0 in "bnz L0", it should go to the first line
fn text_document_definition(
    text_document_position_params: &TextDocumentPositionParams,
) -> Option<Location> {
    // get source
    let source = get_source_from_file(&text_document_position_params.text_document.uri);

    // generate symbol table
    let symbol_table = generate_symbol_map(&source, &text_document_position_params);
    eprintln!("symbol table: {:#?}", symbol_table);

    let word_at_cursor = get_word_at_cursor_from_file(&text_document_position_params);
    let word_at_cursor = word_at_cursor.to_uppercase();

    // return the location of the definition of the word at the cursor
    // which corresponds to the occurrence of the word followed by a colon
    // in the source file

    let definition = match word_at_cursor.strip_suffix(":") {
        Some(def) => def.to_string(),
        None => word_at_cursor,
    };

    // add suffix back so the definition can be searched in the symbol table
    let definition = definition.to_string() + ":";
    eprintln!("definition: {:#?}", definition);

    symbol_table.get(&definition).cloned().or_else(|| {
        // if the definition is not found, return the location it was at anyway
        Some(Location {
            uri: text_document_position_params.text_document.uri.clone(),
            range: Range {
                start: text_document_position_params.position,
                end: text_document_position_params.position,
            },
        })
    })
}

// parses a source file and generates a symbol table
// that maps symbols to their locations
fn generate_symbol_map(
    source: &str,
    text_document_position_params: &TextDocumentPositionParams,
) -> HashMap<String, Location> {
    let mut symbols = HashMap::new();
    for (i, line) in source.lines().enumerate() {
        let mut words = line.split_whitespace();
        if let Some(word) = words.next() {
            symbols.insert(
                word.to_string(),
                Location {
                    uri: Url::from_file_path(uri_to_path(
                        &text_document_position_params.text_document.uri,
                    ))
                    .unwrap(),
                    range: Range {
                        start: Position {
                            line: i as u32,
                            character: 0,
                        },
                        end: Position {
                            line: i as u32,
                            character: word.len() as u32,
                        },
                    },
                },
            );
        }
    }
    symbols
}

fn uri_to_path(uri: &Url) -> String {
    uri.to_file_path().unwrap().to_str().unwrap().to_string()
}

fn get_word_at_cursor_from_file(
    text_document_position_params: &TextDocumentPositionParams,
) -> String {
    let uri = &text_document_position_params.text_document.uri;
    let line = text_document_position_params.position.line as usize;
    let col = text_document_position_params.position.character as usize;

    let filepath = uri.to_file_path().unwrap();

    let file = File::open(filepath).unwrap_or_else(|_| panic!("File not found: {:?}", uri));
    let lines = std::io::BufReader::new(file);
    let line_conts = lines.lines().nth(line).unwrap().unwrap();

    let (start, end) = find_word_at_pos(&line_conts, col);
    line_conts[start..end].to_string()
}

fn find_word_at_pos(line: &str, col: usize) -> (usize, usize) {
    let line_ = format!("{} ", line);
    let is_ident_char = |c: char| c.is_alphanumeric() || c == '_';

    let start = line_
        .chars()
        .enumerate()
        .take(col)
        .filter(|&(_, c)| !is_ident_char(c))
        .last()
        .map(|(i, _)| i + 1)
        .unwrap_or(0);

    #[allow(clippy::filter_next)]
    let mut end = line_
        .chars()
        .enumerate()
        .skip(col)
        .filter(|&(_, c)| !is_ident_char(c));

    let end = end.next();
    (start, end.map(|(i, _)| i).unwrap_or(col))
}

fn get_source_from_file(uri: &Url) -> String {
    let filepath = uri.to_file_path().unwrap();

    let file = File::open(filepath).unwrap_or_else(|_| panic!("File not found: {:?}", uri));
    let lines = std::io::BufReader::new(file);
    lines
        .lines()
        .map(|l| l.unwrap())
        .collect::<Vec<String>>()
        .join("\n")
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

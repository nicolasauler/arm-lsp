use std::error::Error;
use std::fs::File;
use std::io::BufRead;

use lsp_types::request::{GotoDefinition, HoverRequest};
use lsp_types::{
    Hover, HoverContents, HoverProviderCapability, InitializeParams, Location, MarkedString, OneOf,
    Position, Range, ServerCapabilities, TextDocumentPositionParams,
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

fn populate_hashmap() -> Result<HashMap<String, Instruction>, Box<dyn Error + Sync + Send>> {
    let file = std::fs::read_to_string("all.json")?;
    let instructions: Vec<Instruction> = serde_json::from_str(&file)?;

    let mut instructions_map = HashMap::new();
    for instruction in instructions {
        instructions_map.insert(instruction.names[0].clone(), instruction);
    }

    Ok(instructions_map)
}

fn get_instruction<'a>(
    instructions_map: &'a HashMap<String, Instruction>,
    instruction_name: &str,
) -> Option<&'a Instruction> {
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
    instructions_map: &HashMap<String, Instruction>,
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
                        let result =
                            text_document_definition(&params.text_document_position_params);
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
                match cast::<HoverRequest>(req) {
                    Ok((id, params)) => {
                        eprintln!("got hover request #{id}: {params:?}");

                        let word_at_cursor =
                            get_word_at_cursor_from_file(&params.text_document_position_params);
                        eprintln!("word at cursor: {word_at_cursor}");

                        let lsp_types = Hover {
                            contents: HoverContents::Scalar(MarkedString::String(
                                // print operation, summary and symbols
                                get_instruction(&instructions_map, &word_at_cursor)
                                    .unwrap()
                                    .operation
                                    .lines
                                    .join("\n")
                                    + "\n\n"
                                    + &get_instruction(&instructions_map, &word_at_cursor)
                                        .unwrap()
                                        .summary
                                        .lines
                                        .join("\n")
                                    + "\n\n"
                                    + &get_instruction(&instructions_map, &word_at_cursor)
                                        .unwrap()
                                        .symbols
                                        .lines
                                        .join("\n"),
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
fn text_document_definition(
    text_document_position_params: &TextDocumentPositionParams,
) -> Option<Location> {
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

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

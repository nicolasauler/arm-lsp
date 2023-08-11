use std::error::Error;

use lsp_types::request::GotoDefinition;
use lsp_types::{
    InitializeParams, Location, OneOf, ServerCapabilities, TextDocumentPositionParams, Url,
};
use lsp_types::{Position, Range};

use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};

use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    })
    .unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
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

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
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
    let source = get_source(&uri_to_path(
        &text_document_position_params.text_document.uri,
    ))?;

    let symbol_table = SymbolTable::new(&source, text_document_position_params);

    let word = get_word_at_cursor(&source, text_document_position_params);
    let symbol = symbol_table
        .symbols
        .iter()
        .find(|s| s.name == word)
        .unwrap();

    Some(symbol.location.clone())
}

fn get_word_at_cursor(
    source: &str,
    text_document_position_params: &TextDocumentPositionParams,
) -> String {
    let line = source
        .lines()
        .nth(text_document_position_params.position.line as usize)
        .unwrap();
    let mut words = line.split_whitespace();
    let word = words
        .nth(text_document_position_params.position.character as usize)
        .unwrap();
    word.to_string()
}

struct SymbolTable {
    symbols: Vec<Symbol>,
}
impl SymbolTable {
    fn new(source: &str, text_document_position_params: &TextDocumentPositionParams) -> Self {
        let mut symbols = Vec::new();
        for (i, line) in source.lines().enumerate() {
            let mut words = line.split_whitespace();
            if let Some(word) = words.next() {
                symbols.push(Symbol {
                    name: word.to_string(),
                    location: Location {
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
                });
            }
        }
        Self { symbols }
    }
}

struct Symbol {
    name: String,
    location: Location,
}

fn get_source(path: &str) -> Option<String> {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    Some(contents)
}

fn uri_to_path(uri: &Url) -> String {
    uri.to_file_path().unwrap().to_str().unwrap().to_string()
}

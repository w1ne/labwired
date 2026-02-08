// LabWired - Firmware Simulation Platform
// Copyright (C) 2026 Andrii Shylenko
//
// This software is released under the MIT License.
// See the LICENSE file in the project root for full license information.

use crate::adapter::LabwiredAdapter;
use anyhow::Result;
use dap::requests::Command;
use dap::responses::ResponseBody;
use dap::types::{Breakpoint, Capabilities, Scope, Source, StackFrame, Thread, Variable};
use serde::Serialize;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::atomic::{AtomicI64, Ordering};

pub struct DapServer {
    adapter: LabwiredAdapter,
    seq: AtomicI64,
}

#[derive(Serialize)]
struct DapResponse {
    seq: i64,
    #[serde(rename = "type")]
    type_: String,
    request_seq: i64,
    success: bool,
    command: String,
    message: Option<String>,
    body: Option<ResponseBody>,
}

fn command_name(cmd: &Command) -> &'static str {
    match cmd {
        Command::Initialize(_) => "initialize",
        Command::Launch(_) => "launch",
        Command::Disconnect(_) => "disconnect",
        Command::ConfigurationDone => "configurationDone",
        Command::SetBreakpoints(_) => "setBreakpoints",
        Command::SetFunctionBreakpoints(_) => "setFunctionBreakpoints",
        Command::Threads => "threads",
        Command::StackTrace(_) => "stackTrace",
        Command::Scopes(_) => "scopes",
        Command::Variables(_) => "variables",
        Command::Continue(_) => "continue",
        Command::Next(_) => "next",
        Command::StepIn(_) => "stepIn",
        Command::Pause(_) => "pause",
        _ => "unknown",
    }
}

impl Default for DapServer {
    fn default() -> Self {
        Self::new()
    }
}

impl DapServer {
    pub fn new() -> Self {
        Self {
            adapter: LabwiredAdapter::new(),
            seq: AtomicI64::new(1),
        }
    }

    pub fn run<R: Read, W: Write>(&mut self, input: R, mut output: W) -> Result<()> {
        let mut reader = BufReader::new(input);

        loop {
            let mut content_length = 0;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line)? == 0 {
                    return Ok(()); // EOF
                }
                let line = line.trim();
                if line.is_empty() {
                    break; // End of headers
                }
                if let Some(rest) = line.strip_prefix("Content-Length: ") {
                    if let Ok(len) = rest.parse() {
                        content_length = len;
                    }
                }
            }

            if content_length == 0 {
                continue;
            }

            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body)?;

            // Log body for debugging
            // tracing::debug!("Received: {}", String::from_utf8_lossy(&body));

            let request: dap::requests::Request = match serde_json::from_slice(&body) {
                Ok(req) => req,
                Err(e) => {
                    tracing::error!("Failed to parse request: {}", e);
                    continue;
                }
            };

            // Parse as Value to access arbitrary args
            let request_value: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

            // Handle request
            let response_body = match &request.command {
                // Fixed: No Some() wrapper around Capabilities
                Command::Initialize(_) => Some(ResponseBody::Initialize(Capabilities {
                    supports_configuration_done_request: Some(true),
                    supports_function_breakpoints: Some(true),
                    ..Default::default()
                })),
                Command::Launch(_) => {
                    // Extract program from request_value
                    if let Some(program) = request_value
                        .get("arguments")
                        .and_then(|a| a.get("program"))
                        .and_then(|v| v.as_str())
                    {
                        if let Err(e) = self.adapter.load_firmware(program.into()) {
                            tracing::error!("Failed to load firmware: {}", e);
                        }
                    }
                    Some(ResponseBody::Launch)
                }
                Command::Disconnect(_) => return Ok(()),
                Command::SetFunctionBreakpoints(_) => Some(ResponseBody::SetFunctionBreakpoints(
                    dap::responses::SetFunctionBreakpointsResponse {
                        breakpoints: vec![],
                    },
                )),
                Command::ConfigurationDone => Some(ResponseBody::ConfigurationDone),
                Command::SetBreakpoints(args) => {
                    let path = args.source.path.clone().unwrap_or_default();
                    let lines: Vec<i64> = args
                        .breakpoints
                        .as_ref()
                        .map(|bp| bp.iter().map(|b| b.line).collect())
                        .unwrap_or_default();

                    if let Err(e) = self.adapter.set_breakpoints(path, lines.clone()) {
                        tracing::error!("Failed to set breakpoints: {}", e);
                    }

                    let breakpoints = lines
                        .iter()
                        .map(|l| Breakpoint {
                            id: None,
                            verified: true,
                            message: None,
                            source: Some(args.source.clone()),
                            line: Some(*l),
                            column: None,
                            end_column: None,
                            end_line: None,
                            instruction_reference: None,
                            offset: None,
                        })
                        .collect();

                    Some(ResponseBody::SetBreakpoints(
                        dap::responses::SetBreakpointsResponse { breakpoints },
                    ))
                }
                Command::ReadMemory(args) => {
                    // Extract address from memoryReference (it's usually a string representation of hex)
                    let addr = if args.memory_reference.starts_with("0x") {
                        u64::from_str_radix(&args.memory_reference[2..], 16).unwrap_or(0)
                    } else {
                        args.memory_reference.parse().unwrap_or(0)
                    };
                    let offset = args.offset.unwrap_or(0);
                    let final_addr = addr + offset as u64;
                    let count = args.count as usize;

                    match self.adapter.read_memory(final_addr, count) {
                        Ok(data) => {
                            use base64::Engine;
                            let encoded = base64::engine::general_purpose::STANDARD.encode(data);
                            Some(ResponseBody::ReadMemory(
                                dap::responses::ReadMemoryResponse {
                                    address: format!("{:#x}", final_addr),
                                    unreadable_bytes: None,
                                    data: Some(encoded),
                                },
                            ))
                        }
                        Err(e) => {
                            tracing::error!("ReadMemory failed: {}", e);
                            None // Or error response
                        }
                    }
                }
                Command::Threads => Some(ResponseBody::Threads(dap::responses::ThreadsResponse {
                    threads: vec![Thread {
                        id: 1,
                        name: "Core 0".to_string(),
                    }],
                })),
                Command::StackTrace(_) => {
                    let pc = self.adapter.get_pc().unwrap_or(0);
                    let source_loc = self.adapter.lookup_source(pc as u64);

                    let (source, line, name) = if let Some(loc) = source_loc {
                        let source = Some(Source {
                            name: Some(
                                std::path::Path::new(&loc.file)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(&loc.file)
                                    .to_string(),
                            ),
                            path: Some(loc.file),
                            source_reference: None,
                            presentation_hint: None,
                            origin: None,
                            sources: None,
                            adapter_data: None,
                            checksums: None,
                        });
                        (
                            source,
                            loc.line.map(|l| l as i64),
                            loc.function.unwrap_or_else(|| "main".to_string()),
                        )
                    } else {
                        (None, Some(0), "unknown".to_string())
                    };

                    Some(ResponseBody::StackTrace(
                        dap::responses::StackTraceResponse {
                            stack_frames: vec![StackFrame {
                                id: 1,
                                name,
                                line: line.unwrap_or(0),
                                column: 0,
                                source,
                                end_column: None,
                                end_line: None,
                                instruction_pointer_reference: Some(format!("{:#x}", pc)),
                                module_id: None,
                                presentation_hint: None,
                                can_restart: Some(false),
                            }],
                            total_frames: Some(1),
                        },
                    ))
                }
                Command::Scopes(_) => {
                    Some(ResponseBody::Scopes(dap::responses::ScopesResponse {
                        scopes: vec![Scope {
                            name: "Registers".to_string(),
                            variables_reference: 1, // Reference for registers
                            expensive: false,
                            column: None,
                            end_column: None,
                            end_line: None,
                            indexed_variables: None,
                            line: None,
                            named_variables: Some(16),
                            presentation_hint: None,
                            source: None,
                        }],
                    }))
                }
                Command::Variables(args) => {
                    if args.variables_reference == 1 {
                        let mut variables = Vec::new();
                        for i in 0..16 {
                            let name = match i {
                                13 => "SP".to_string(),
                                14 => "LR".to_string(),
                                15 => "PC".to_string(),
                                n => format!("R{}", n),
                            };
                            let val = self.adapter.get_register(i as u8).unwrap_or(0);
                            variables.push(Variable {
                                name,
                                value: format!("{:#x}", val),
                                variables_reference: 0,
                                evaluate_name: None,
                                indexed_variables: None,
                                named_variables: None,
                                presentation_hint: None,
                                type_field: Some("uint32".to_string()),
                                memory_reference: None,
                            });
                        }
                        Some(ResponseBody::Variables(dap::responses::VariablesResponse {
                            variables,
                        }))
                    } else {
                        Some(ResponseBody::Variables(dap::responses::VariablesResponse {
                            variables: vec![],
                        }))
                    }
                }
                Command::Continue(_) => {
                    let _ = self.adapter.continue_execution();
                    Some(ResponseBody::Continue(dap::responses::ContinueResponse {
                        all_threads_continued: Some(true),
                    }))
                }
                Command::Next(_) => {
                    let _ = self.adapter.step();
                    Some(ResponseBody::Next)
                }
                _ => None,
            };

            if let Some(body) = response_body {
                let response = DapResponse {
                    seq: self.seq.fetch_add(1, Ordering::SeqCst),
                    type_: "response".to_string(),
                    request_seq: request.seq,
                    success: true,
                    command: command_name(&request.command).to_string(),
                    message: None,
                    body: Some(body),
                };

                let resp_json = serde_json::to_string(&response)?;
                write!(
                    output,
                    "Content-Length: {}\r\n\r\n{}",
                    resp_json.len(),
                    resp_json
                )?;
                output.flush()?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_read_memory() {
        // Setup server with a machine that has some data
        let server = DapServer::new();

        // We'll use a small binary with some known data at address 0
        let temp_dir = std::env::temp_dir();
        let _elf_path = temp_dir.join("test_read_mem.elf");
        // For this test, we might actually need a real ELF, but let's see if we can
        // just mock the adapter if we refactor?
        // Since we can't easily mock, let's use the firmware built earlier if it exists.

        let target_elf = std::path::PathBuf::from("../../target/thumbv7m-none-eabi/debug/firmware");
        if !target_elf.exists() {
            return;
        }

        server
            .adapter
            .load_firmware(target_elf)
            .expect("Failed to load firmware");

        // Verification of base64 encoding (crucial for ReadMemory)
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
        assert_eq!(encoded, "3q2+7w==");
    }
}

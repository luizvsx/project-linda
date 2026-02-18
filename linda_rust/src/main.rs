use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, Condvar};
use std::thread;

const PORT: &str = "127.0.0.1:54321";

struct SharedState {
    data: HashMap<String, VecDeque<String>>,
}

type SharedTupleSpace = Arc<(Mutex<SharedState>, Condvar)>;

// =====================
// Serviços
// =====================

fn apply_service(id: u32, input: &str) -> Option<String> {
    match id {
        1 => Some(input.to_uppercase()),
        2 => Some(input.chars().rev().collect()),
        3 => Some(input.len().to_string()),
        _ => None,
    }
}

// =====================
// Operações bloqueantes
// =====================

fn wait_and_peek(shared: &SharedTupleSpace, key: &str) -> String {
    let (lock, cvar) = &**shared;
    let mut state = lock.lock().unwrap();

    loop {
        if let Some(queue) = state.data.get(key) {
            if let Some(value) = queue.front() {
                return value.clone();
            }
        }

        state = cvar.wait(state).unwrap();
    }
}

fn wait_and_pop(shared: &SharedTupleSpace, key: &str) -> String {
    let (lock, cvar) = &**shared;
    let mut state = lock.lock().unwrap();

    loop {
        if state.data.contains_key(key) {

            let should_remove;
            let value;

            {
                let queue = state.data.get_mut(key).unwrap();

                if let Some(v) = queue.pop_front() {
                    value = v;
                    should_remove = queue.is_empty();
                } else {
                    state = cvar.wait(state).unwrap();
                    continue;
                }
            }

            if should_remove {
                state.data.remove(key);
            }

            return value;
        }

        state = cvar.wait(state).unwrap();
    }
}

// =====================
// Cliente
// =====================

fn handle_client(mut stream: TcpStream, shared: SharedTupleSpace) {
    let reader = BufReader::new(stream.try_clone().unwrap());

    for line in reader.lines() {
        let command = match line {
            Ok(cmd) => cmd,
            Err(_) => break,
        };

        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            let _ = stream.write_all(b"ERROR\n");
            continue;
        }

        match parts[0] {

            // WR chave valor
            "WR" if parts.len() >= 3 => {
                let key = parts[1].to_string();
                let value = parts[2..].join(" ");

                let (lock, cvar) = &*shared;
                let mut state = lock.lock().unwrap();

                let queue = state.data.entry(key).or_insert(VecDeque::new());
                queue.push_back(value);

                cvar.notify_all();

                let _ = stream.write_all(b"OK\n");
            }

            // RD chave
            "RD" if parts.len() == 2 => {
                let key = parts[1];
                let value = wait_and_peek(&shared, key);
                let response = format!("OK {}\n", value);
                let _ = stream.write_all(response.as_bytes());
            }

            // IN chave
            "IN" if parts.len() == 2 => {
                let key = parts[1];
                let value = wait_and_pop(&shared, key);
                let response = format!("OK {}\n", value);
                let _ = stream.write_all(response.as_bytes());
            }

            // EX k_in k_out svc_id
            "EX" if parts.len() == 4 => {

                let k_in = parts[1];
                let k_out = parts[2];

                let svc_id: u32 = match parts[3].parse() {
                    Ok(id) => id,
                    Err(_) => {
                        let _ = stream.write_all(b"ERROR\n");
                        continue;
                    }
                };

                let value = wait_and_pop(&shared, k_in);

                match apply_service(svc_id, &value) {

                    Some(result) => {
                        let (lock, cvar) = &*shared;
                        let mut state = lock.lock().unwrap();

                        let queue = state.data
                            .entry(k_out.to_string())
                            .or_insert(VecDeque::new());

                        queue.push_back(result);

                        cvar.notify_all();

                        let _ = stream.write_all(b"OK\n");
                    }

                    None => {
                        let _ = stream.write_all(b"NO-SERVICE\n");
                    }
                }
            }

            _ => {
                let _ = stream.write_all(b"ERROR\n");
            }
        }
    }
}

// =====================
// Main
// =====================

fn main() {
    let listener = TcpListener::bind(PORT)
        .expect("Não conseguiu abrir a porta");

    println!("Servidor rodando na porta 54321...");

    let shared: SharedTupleSpace = Arc::new((
        Mutex::new(SharedState {
            data: HashMap::new(),
        }),
        Condvar::new(),
    ));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let shared_clone = Arc::clone(&shared);

                thread::spawn(|| {
                    handle_client(stream, shared_clone);
                });
            }
            Err(e) => {
                eprintln!("Erro na conexão: {}", e);
            }
        }
    }
}

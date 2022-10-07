use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    println!("Starting the server...");

    // Servidor TCP que recebe as conexões e mensagens dos clientes
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    // ** O que é [non]blocking?
    server.set_nonblocking(true).expect("Failed to initialize non-blocking");

    // Vetor para armazenar clientes conectados ao servidor
    let mut clients = vec![];
    // Transmissor e Receptor de mensagens recebidas na thread para o servidor fora da thread
    let (tx, rx) = mpsc::channel::<String>();

    loop {
        // Se for aceita com sucesso uma nova conexão com um cliente
        if let Ok( (mut socket, addr) ) = server.accept() {
            println!("Client {} connected", addr);
            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone client"));

            // Para cada conexão com cliente, cria uma thread
            thread::spawn(move || loop {
                // Cria um buffer para armazenar as mensagens enviadas pelo cliente
                let mut buff = vec![0; MSG_SIZE];
                // Lê a mensagem enviada pelo cliente
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        // Itera sobre a mensagem, [u8], e
                        // remove do buffer a parte que não foi utilizada com a mensagem
                        let msg = buff
                            .into_iter()
                            .take_while(|&x| x != 0)
                            .collect::<Vec<_>>();

                        let msg = String::from_utf8(msg).expect("Invalid UTF-8 message");
                        println!("{}: {}", addr, msg);
                        // ** Envia para o servidor fora da thread
                        tx.send(msg).expect("Failed to send message to rx");
                    },
                    Err(err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with {}", addr);
                        break;
                    },
                }
                sleep();
            });
        }

        // Para cada mensagem recebida de um cliente (dentro da thread), o
        // servidor redimensiona essa mensagem e envia para todos os clientes
        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }
        sleep();
    }
}

fn sleep() {
    thread::sleep(Duration::from_millis(100));
}
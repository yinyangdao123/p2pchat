use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{stdin, Read, Write};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;
use chapter7::app::App;

pub fn register_name(mut stream:TcpStream, local_name:String) {
    let msg = format!("0{}", local_name);
    stream.write_all(msg.as_bytes()).unwrap();
}


fn handle_stream(mut stream:TcpStream, app:Arc<Mutex<App>>) {
    let mut buffer = [0; 512];
    let remote_address = stream.peer_addr().unwrap();
    loop {
        sleep(Duration::from_millis(500));
        
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 { 
                    println!("{} disconnected", remote_address);
                    {app.lock().unwrap().contact_list_remove_by_address(&remote_address);}
                    break; 
                }
                match str::from_utf8(&[buffer[0]]) {
                    Ok("0") => {
                        let remote_name = str::from_utf8(&buffer[1..bytes_read]).unwrap().trim().to_string();
                        println!("{} as name {}", remote_address, remote_name);
                        {app.lock().unwrap().contact_list_insert_name_address(remote_name, remote_address);}
                    },
                    Ok("1") =>{
                        if let Some(remote_name) = {app.lock().unwrap().contact_list_get_name_by_address(&remote_address)} {
                            println!("[{}]: {}", remote_name, str::from_utf8(&buffer[1..bytes_read]).unwrap().trim());
                        }
                        else {
                            println!("[{}]: {}", remote_address, str::from_utf8(&buffer[1..bytes_read]).unwrap().trim());
                        }
                    },
                    _=>{}
                }
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                //什么也不做，让循环继续尝试
            }
            Err(_) => {
                println!("{} disconnected", remote_address);
                {app.lock().unwrap().contact_list_remove_by_address(&remote_address);}
                break;
            },
        }
    }
}


fn bind(app:Arc<Mutex<App>>) {
    let local_address = {app.lock().unwrap().get_local_address()};
    let listener = TcpListener::bind(local_address).unwrap();
    println!("Listening on: {}", listener.local_addr().unwrap());

    for stream in listener.incoming() {
        match stream {
            Ok(stream)=>{
                stream.set_nonblocking(true).unwrap();

                println!("{} connected", stream.peer_addr().unwrap());

                let local_name = {app.lock().unwrap().get_local_name()};
                register_name(stream.try_clone().unwrap(), local_name);
                
                let remote_address = stream.peer_addr().unwrap();
                let stream_clone = stream.try_clone().unwrap();
                
                {app.lock().unwrap().contact_list_insert_address_stream(remote_address, stream);}

                let app_clone = app.clone();
                thread::spawn(move || {
                    handle_stream(stream_clone, app_clone);
                });
            }
            Err(_)=>{
                break;
            }
        }
    }
}

fn connect(remote_address: SocketAddr, app:Arc<Mutex<App>>){
    let stream = TcpStream::connect(remote_address).unwrap();
    stream.set_nonblocking(true).unwrap();

    println!("{} connected", stream.peer_addr().unwrap());

    let local_name = {app.lock().unwrap().get_local_name()};
    register_name(stream.try_clone().unwrap(), local_name);
    
    let remote_address = stream.peer_addr().unwrap();
    let stream_clone = stream.try_clone().unwrap();
    
    {app.lock().unwrap().contact_list_insert_address_stream(remote_address, stream);}
                
    let app_clone = app.clone();
    thread::spawn(move || {
        handle_stream(stream_clone, app_clone);
    });
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    let (_, Some(local_address), Some(name), None) = (args.next(), args.next(), args.next(), args.next())
    else {
        return Err(std::io::Error::other("Please run [peer ip:port name]"));
    };
    let local_address = SocketAddr::from_str(local_address.as_str().trim()).unwrap();
    let local_name = name.trim().to_string();

    let app = Arc::new(Mutex::new(App::new(local_address, local_name)));
    let app_bind = app.clone();

    thread::spawn(move ||{
        bind(app_bind);
    });

    loop{
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        if input.len() == 0 {continue;}
        let (input_0, input_1) = input.trim().split_once(char::is_whitespace).unwrap_or((input.as_str().trim(), ""));
        let (input_0, input_1)= (input_0.trim(), input_1.trim());
        
        match input_0{
            "connect" =>{
                match SocketAddr::from_str(input_1.replace(char::is_whitespace, "").as_str()) {
                    Ok(remote_address)=>{
                        connect(remote_address, app.clone());
                    }
                    Err(_)=>{
                        println!("Please run [connect ip:port]");
                        continue;
                    }
                }
            }
            "quit" => {
                break;
            }
            "message" => {
                if let Some((remote_address_or_name, content)) = input_1.split_once(char::is_whitespace) {
                    let (remote_address_or_name, content) = (remote_address_or_name.trim(), content.trim());
                    let msg = format!("1{}", content);
                    if let Ok(remote_address) = SocketAddr::from_str(remote_address_or_name.replace(char::is_whitespace, "").as_str()) {
                        if let Some(mut stream) = {app.lock().unwrap().contact_list_get_stream_by_address(&remote_address)} {
                            stream.write_all(msg.as_bytes())?;
                        }
                    }
                    else if let Some(mut stream) = {app.lock().unwrap().contact_list_get_stream_by_name(&remote_address_or_name.to_string())} {
                        stream.write_all(msg.as_bytes())?;
                    }
                    else {
                        println!("Please run [message ip:port|remote_name content]");
                        continue;
                    }
                }
            }
            "list" => {
                {app.lock().unwrap().contact_list_display();}
            }
            _=>{}
        }
    }
    Ok(())
}

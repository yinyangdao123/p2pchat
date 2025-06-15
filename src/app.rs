use std::{collections::HashMap, net::{SocketAddr, TcpStream}};

pub struct App {
    local_address:SocketAddr,
    local_name:String,
    contact_list_address_stream:HashMap<SocketAddr, TcpStream>,
    contact_list_name_address:HashMap<String, SocketAddr>
}

impl App {
    pub fn new(local_address:SocketAddr, local_name:String) -> Self {
        App {
            local_address,
            local_name,
            contact_list_address_stream:HashMap::new(),
            contact_list_name_address:HashMap::new()
        }
    }

    pub fn get_local_address(&self)->SocketAddr{
        self.local_address.clone()
    }

    pub fn get_local_name(&self)->String{
        self.local_name.clone()
    }

    pub fn contact_list_get_address_by_name(&self, remote_name:&String)->Option<SocketAddr>{
        match self.contact_list_name_address.get(remote_name) {
            Some(value) => Some(value.to_owned()),
            None=>None
        }
    }

    pub fn contact_list_get_stream_by_address(&self, remote_address:&SocketAddr)->Option<TcpStream>{
        match self.contact_list_address_stream.get(remote_address) {
            Some(value) => Some(value.try_clone().unwrap()),
            None=>None
        }
    }

    pub fn contact_list_get_stream_by_name(&self, remote_name:&String)->Option<TcpStream>{
        match self.contact_list_get_address_by_name(remote_name) {
            Some(remote_address)=>self.contact_list_get_stream_by_address(&remote_address),
            None=>None
        }
    }

    pub fn contact_list_get_name_by_address(&self, remote_address:&SocketAddr)->Option<String>{
        let mut filter = self.contact_list_name_address
        .iter()
        .filter(|x|x.1==remote_address);

        match filter.next() {
            Some(value)=>Some(value.0.clone()),
            None=>None
        }
    }

    pub fn contact_list_remove_by_address(&mut self, remote_address:&SocketAddr) {
        self.contact_list_address_stream.remove(remote_address);
    }

    pub fn contact_list_remove_by_name(&mut self, remote_name:String) {
        if let Some(remote_address) = self.contact_list_get_address_by_name(&remote_name) {
            self.contact_list_remove_by_address(&remote_address);
        }
        self.contact_list_name_address.remove(&remote_name);
    }

    pub fn contact_list_insert_address_stream(&mut self, remote_address:SocketAddr, stream:TcpStream){
        self.contact_list_address_stream.insert(remote_address, stream);
    }

    pub fn contact_list_insert_name_address(&mut self, remote_name:String, remote_address:SocketAddr){
        self.contact_list_name_address.insert(remote_name, remote_address);
    }

    pub fn contact_list_display(&self) {
        self.contact_list_address_stream
        .iter()
        .for_each(|x|{
            let remote_address = x.0;
            if let Some(remote_name) = self.contact_list_get_name_by_address(remote_address) {
                println!("{} -> {}", remote_address, remote_name);  
            }
            else {
                println!("{} -> {}",remote_address, "")
            }
        });
    }

}
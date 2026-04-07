use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{Read, Write};

pub struct TcpServer {
    listener: Option<TcpListener>,
    port: u16,
}

impl TcpServer {
    pub fn new(port: u16) -> Self {
        TcpServer { listener: None, port }
    }
    
    pub fn bind(&mut self, addr: &str) -> std::io::Result<()> {
        self.listener = Some(TcpListener::bind(addr)?);
        Ok(())
    }
    
    pub fn accept(&mut self) -> std::io::Result<(TcpStream, SocketAddr)> {
        if let Some(ref listener) = self.listener {
            listener.accept()
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "Server not bound"))
        }
    }
    
    pub fn listen(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct TcpClient {
    stream: Option<TcpStream>,
}

impl TcpClient {
    pub fn connect(addr: &str) -> std::io::Result<Self> {
        Ok(TcpClient { stream: Some(TcpStream::connect(addr)?) })
    }
    
    pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(ref mut stream) = self.stream {
            stream.read(buf)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "Not connected"))
        }
    }
    
    pub fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(ref mut stream) = self.stream {
            stream.write(buf)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "Not connected"))
        }
    }
}

pub fn resolve_host(host: &str, port: u16) -> std::io::Result<SocketAddr> {
    let addr = format!("{}:{}", host, port);
    addr.parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
}

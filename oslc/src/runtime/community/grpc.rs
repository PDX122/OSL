use std::collections::HashMap;

pub struct GrpcServer {
    services: HashMap<String, GrpcService>,
    port: u16,
}

struct GrpcService {
    name: String,
    methods: HashMap<String, GrpcMethod>,
}

struct GrpcMethod {
    input: String,
    output: String,
    handler: Option<Box<dyn Fn(&[u8]) -> Vec<u8>>>,
}

impl GrpcServer {
    pub fn new(port: u16) -> Self {
        GrpcServer {
            services: HashMap::new(),
            port,
        }
    }
    
    pub fn add_service(&mut self, name: &str) -> &mut GrpcService {
        let service = GrpcService {
            name: name.to_string(),
            methods: HashMap::new(),
        };
        self.services.insert(name.to_string(), service);
        self.services.get_mut(name).unwrap()
    }
    
    pub fn start(&self) -> Result<(), String> {
        Ok(())
    }
}

impl GrpcService {
    pub fn method(&mut self, name: &str, input: &str, output: &str) {
        self.methods.insert(name.to_string(), GrpcMethod {
            input: input.to_string(),
            output: output.to_string(),
            handler: None,
        });
    }
}

pub struct GrpcClient {
    host: String,
    port: u16,
}

impl GrpcClient {
    pub fn new(host: &str, port: u16) -> Self {
        GrpcClient {
            host: host.to_string(),
            port,
        }
    }
    
    pub fn call(&self, _service: &str, _method: &str, _data: &[u8]) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }
}

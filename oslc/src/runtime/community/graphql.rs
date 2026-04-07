pub struct GraphQLSchema {
    query_type: Option<String>,
    mutation_type: Option<String>,
}

impl GraphQLSchema {
    pub fn new() -> Self {
        GraphQLSchema {
            query_type: None,
            mutation_type: None,
        }
    }
    
    pub fn query(&mut self, name: &str) -> &mut Self {
        self.query_type = Some(name.to_string());
        self
    }
    
    pub fn mutation(&mut self, name: &str) -> &mut Self {
        self.mutation_type = Some(name.to_string());
        self
    }
    
    pub fn field(&mut self, _name: &str, _ty: &str) -> &mut Self {
        self
    }
}

pub struct GraphQLResolver {
    _schema: GraphQLSchema,
}

impl GraphQLResolver {
    pub fn new() -> Self {
        GraphQLResolver {
            _schema: GraphQLSchema::new(),
        }
    }
    
    pub fn resolve(&self, _query: &str) -> Result<String, String> {
        Ok("{}".to_string())
    }
}

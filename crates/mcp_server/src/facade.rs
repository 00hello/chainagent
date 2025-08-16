use domain::*;

pub struct ServerFacade<T: Toolbox> {
    toolbox: T,
}

impl<T: Toolbox> ServerFacade<T> {
    pub fn new(toolbox: T) -> Self { Self { toolbox } }
    pub fn toolbox(&self) -> &T { &self.toolbox }
}


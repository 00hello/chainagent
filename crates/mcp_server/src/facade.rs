use domain::*;

#[allow(dead_code)]
pub struct ServerFacade<T: Toolbox> {
    toolbox: T,
}

#[allow(dead_code)]
impl<T: Toolbox> ServerFacade<T> {
    pub fn new(toolbox: T) -> Self { Self { toolbox } }
    pub fn toolbox(&self) -> &T { &self.toolbox }
}


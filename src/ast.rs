use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct Program {
    pub ast_nodes: Vec<AstNode>,
}

#[derive(Debug)]
pub enum AstNode {
    Sequence(Sequences),
    Mediator(Mediators),
}

#[derive(Debug)]
pub enum Sequences {
    InSequence(InSequence),
}

#[derive(Debug)]
pub enum Mediators {
    Log(LogMediator),
    Property(PropertyMediator),
}

//--------------------------------------------------------------------------------//
#[derive(Debug)]
pub struct InSequence {
    pub mediators: Vec<Mediators>,
}

#[derive(Debug)]
pub struct LogMediator {
    pub level: String,
    pub properties: Vec<PropertyMediator>,
}

#[derive(Debug)]
pub struct PropertyMediator {
    pub name: String,
    pub value: String,
}

//--------------------------------------------------------------------------------//
impl IntoIterator for Program {
    type Item = AstNode;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.ast_nodes.into_iter()
    }
}

//--------------------------------------------------------------------------------//
impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for ast_node in &self.ast_nodes {
            write!(f, "{}", ast_node)?;
        }
        Ok(())
    }
}

impl Display for AstNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AstNode::Sequence(sequence) => write!(f, "{}", sequence),
            AstNode::Mediator(mediator) => write!(f, "{}", mediator),
        }
    }
}

impl Display for Sequences {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Sequences::InSequence(in_sequence) => write!(f, "{}", in_sequence),
        }
    }
}

impl Display for InSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<inSequence>")?;
        for mediator in &self.mediators {
            write!(f, "{}", mediator)?;
        }
        write!(f, "</inSequence>")
    }
}

impl Display for Mediators {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Mediators::Log(log_mediator) => write!(f, "{}", log_mediator),
            Mediators::Property(property_mediator) => write!(f, "{}", property_mediator),
        }
    }
}

impl Display for LogMediator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<log level=\"{}\">", self.level)?;
        for property in &self.properties {
            write!(f, "{}", property)?;
        }
        write!(f, "</log>")
    }
}

impl Display for PropertyMediator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<property name=\"{}\" value=\"{}\"/>",
            self.name, self.value
        )
    }
}
